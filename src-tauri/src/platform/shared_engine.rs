//! Windows: one WebView2 browser shared by Cloister and every service.
//!
//! By default each WebView2 data folder starts its own browser, GPU and
//! network processes. Cloister instead takes the main window's WebView2
//! environment and creates every service inside it as a separate *profile*
//! (see `vendor/README.md`). Profiles keep cookies and storage apart, while
//! the expensive browser plumbing exists only once.
//! See <https://learn.microsoft.com/microsoft-edge/webview2/concepts/multi-profile-support>.
//!
//! Because services now share processes, memory can no longer be measured per
//! folder. Instead WebView2 reports which frames each process renders, and each
//! service's main frame id (recorded after its first page load) ties renderer
//! processes back to services.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;

use tauri::webview::PlatformWebview;
use tauri::{AppHandle, Runtime, Webview};
use webview2_com::GetProcessExtendedInfosCompletedHandler;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Environment, ICoreWebView2Environment13, ICoreWebView2FrameInfo, ICoreWebView2FrameInfo2,
    ICoreWebView2ProcessExtendedInfoCollection, ICoreWebView2_20, COREWEBVIEW2_FRAME_KIND_MAIN_FRAME,
};
use windows_core::Interface;

/// The shared environment. COM reference counting is thread-safe, so the
/// pointer may be cloned on any thread; it is only *used* on the UI thread,
/// where Tauri creates webviews. Tauri relies on the same reasoning for its
/// own `WebviewAttributes::environment`.
struct SharedEnvironment(ICoreWebView2Environment);
// SAFETY: see the type's documentation above.
unsafe impl Send for SharedEnvironment {}
// SAFETY: the value is never mutated after it is set.
unsafe impl Sync for SharedEnvironment {}

static ENVIRONMENT: OnceLock<SharedEnvironment> = OnceLock::new();

/// Main frame id of each service webview, by label.
static MAIN_FRAMES: Mutex<Option<HashMap<String, u32>>> = Mutex::new(None);

/// How long to wait for WebView2 to list its processes.
const PROCESS_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Records the main window's environment so services can join it. Call once
/// at startup; the environment is available a moment later.
pub fn capture<R: Runtime>(main: &Webview<R>) {
    let result = main.with_webview(|platform| {
        let _ = ENVIRONMENT.set(SharedEnvironment(platform.environment()));
        log::info!("services will share the main WebView2 engine");
    });
    if let Err(err) = result {
        log::warn!("could not reach the main webview to share its engine: {err}");
    }
}

/// The shared environment, once captured.
pub fn environment() -> Option<ICoreWebView2Environment> {
    ENVIRONMENT.get().map(|shared| shared.0.clone())
}

/// Remembers a service webview's main frame id. Call after a page load: the
/// id is only valid once the webview has started navigating.
pub fn remember_main_frame<R: Runtime>(webview: &Webview<R>) {
    let label = webview.label().to_string();
    let _ = webview.with_webview(move |platform| match main_frame_id(&platform) {
        Ok(id) => {
            let mut frames = MAIN_FRAMES.lock().unwrap_or_else(|e| e.into_inner());
            frames.get_or_insert_with(HashMap::new).insert(label, id);
        }
        Err(err) => log::debug!("no main frame id for {label}: {err}"),
    });
}

/// Forgets a closed webview's main frame id.
pub fn forget_main_frame(label: &str) {
    let mut frames = MAIN_FRAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(frames) = frames.as_mut() {
        frames.remove(label);
    }
}

fn main_frame_id(platform: &PlatformWebview) -> windows_core::Result<u32> {
    let mut id = 0;
    // SAFETY: plain COM calls on a live controller, on its UI thread (`with_webview`).
    unsafe {
        let core = platform.controller().CoreWebView2()?;
        core.cast::<ICoreWebView2_20>()?.FrameId(&mut id)?;
    }
    Ok(id)
}

/// One browser process and the main frames whose content it renders.
#[derive(Debug, Clone, Default)]
pub struct EngineProcess {
    pub pid: u32,
    pub main_frames: Vec<u32>,
}

/// Memory of the shared engine split by service label, plus everything that
/// belongs to no single service (browser, GPU and network processes, and
/// Cloister's own window).
#[derive(Debug, Clone, Default)]
pub struct EngineUsage {
    pub by_label: HashMap<String, u64>,
    pub shared: u64,
}

/// Lists the engine's processes on the UI thread and splits their memory by
/// service. `memory_of` returns a process's resident memory in bytes.
pub fn usage<R: Runtime>(app: &AppHandle<R>, memory_of: impl Fn(u32) -> u64) -> Option<EngineUsage> {
    let (tx, rx) = mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = tx.send(list_processes());
    })
    .ok()?;
    let processes = match rx.recv_timeout(PROCESS_QUERY_TIMEOUT) {
        Ok(Ok(processes)) => processes,
        Ok(Err(err)) => {
            log::debug!("could not list WebView2 processes: {err}");
            return None;
        }
        Err(_) => return None,
    };

    let frames = MAIN_FRAMES.lock().unwrap_or_else(|e| e.into_inner()).clone().unwrap_or_default();
    let label_of_frame: HashMap<u32, &String> = frames.iter().map(|(label, id)| (*id, label)).collect();

    let mut usage = EngineUsage::default();
    for process in processes {
        let bytes = memory_of(process.pid);
        // A renderer serves one profile, so its frames all belong to one service.
        match process.main_frames.iter().find_map(|id| label_of_frame.get(id)) {
            Some(label) => *usage.by_label.entry((*label).clone()).or_default() += bytes,
            None => usage.shared += bytes,
        }
    }
    Some(usage)
}

/// Asks WebView2 for every process of the shared engine. UI thread only.
fn list_processes() -> windows_core::Result<Vec<EngineProcess>> {
    let Some(environment) = environment() else { return Ok(Vec::new()) };
    let environment = environment.cast::<ICoreWebView2Environment13>()?;
    let collected: Rc<RefCell<windows_core::Result<Vec<EngineProcess>>>> =
        Rc::new(RefCell::new(Ok(Vec::new())));
    let sink = Rc::clone(&collected);

    GetProcessExtendedInfosCompletedHandler::wait_for_async_operation(
        // SAFETY: COM call on the UI thread that owns the environment.
        Box::new(move |handler| unsafe { environment.GetProcessExtendedInfos(&handler).map_err(Into::into) }),
        Box::new(move |result, collection| {
            *sink.borrow_mut() = result.and_then(|()| match collection {
                Some(collection) => read_collection(&collection),
                None => Ok(Vec::new()),
            });
            Ok(())
        }),
    )
    .map_err(|err| windows_core::Error::new(windows_core::HRESULT(-1), format!("{err:?}")))?;

    Rc::try_unwrap(collected).map(RefCell::into_inner).unwrap_or_else(|rc| rc.borrow().clone())
}

fn read_collection(
    collection: &ICoreWebView2ProcessExtendedInfoCollection,
) -> windows_core::Result<Vec<EngineProcess>> {
    let mut processes = Vec::new();
    // SAFETY: COM calls on objects handed to the completion handler, on the UI thread.
    unsafe {
        let mut count = 0;
        collection.Count(&mut count)?;
        for index in 0..count {
            let info = collection.GetValueAtIndex(index)?;
            let mut pid = 0;
            info.ProcessInfo()?.ProcessId(&mut pid)?;
            let mut main_frames = Vec::new();
            if let Ok(frames) = info.AssociatedFrameInfos() {
                let iterator = frames.GetIterator()?;
                let mut has_current = windows_core::BOOL::default();
                iterator.HasCurrent(&mut has_current)?;
                while has_current.as_bool() {
                    if let Some(id) = iterator.GetCurrent().ok().and_then(|frame| main_frame_of(&frame)) {
                        main_frames.push(id);
                    }
                    iterator.MoveNext(&mut has_current)?;
                }
            }
            processes.push(EngineProcess { pid: u32::try_from(pid).unwrap_or_default(), main_frames });
        }
    }
    Ok(processes)
}

/// Walks up from any frame to its page's main frame and returns that id.
fn main_frame_of(frame: &ICoreWebView2FrameInfo) -> Option<u32> {
    let mut current = frame.cast::<ICoreWebView2FrameInfo2>().ok()?;
    // Pages nest frames only a few levels deep; the bound guards against cycles.
    for _ in 0..16 {
        // SAFETY: COM calls on a frame info object, on the UI thread.
        unsafe {
            let mut kind = Default::default();
            current.FrameKind(&mut kind).ok()?;
            if kind == COREWEBVIEW2_FRAME_KIND_MAIN_FRAME {
                let mut id = 0;
                current.FrameId(&mut id).ok()?;
                return Some(id);
            }
            current = current.ParentFrameInfo().ok()?.cast::<ICoreWebView2FrameInfo2>().ok()?;
        }
    }
    None
}
