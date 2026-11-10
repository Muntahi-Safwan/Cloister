//! Linux: places service pages over the main UI.
//!
//! On Linux, Tauri packs every webview of a window into one vertical `GtkBox`,
//! so service pages split the window with Cloister's own UI and ignore
//! `set_bounds`. This module wraps the main UI in a `GtkOverlay` and moves
//! service pages into a `GtkFixed` layer on top of it. The layer is placed
//! with margins and sized by its visible child, so it only covers the content
//! area and the sidebar and title bar keep receiving clicks.
//!
//! GTK objects must stay on the main thread: the layer lives in a
//! thread-local, and every function below touches GTK only from `setup` or
//! inside `with_webview`, both of which run on the main thread.

use std::cell::RefCell;

use gtk::prelude::*;
use tauri::{Runtime, Webview, Window};

use crate::state::ContentBounds;

thread_local! {
    static LAYER: RefCell<Option<gtk::Fixed>> = const { RefCell::new(None) };
}

/// Wraps the main UI in an overlay with an empty layer for service pages.
/// Call once from `setup`, after the main window exists.
pub fn install<R: Runtime>(window: &Window<R>) {
    let Ok(vbox) = window.default_vbox() else { return };
    let Some(main_ui) = vbox.children().first().cloned() else { return };

    let overlay = gtk::Overlay::new();
    vbox.remove(&main_ui);
    overlay.add(&main_ui);

    let layer = gtk::Fixed::new();
    layer.set_halign(gtk::Align::Start);
    layer.set_valign(gtk::Align::Start);
    overlay.add_overlay(&layer);

    vbox.pack_start(&overlay, true, true, 0);
    overlay.show_all();
    LAYER.with(|cell| *cell.borrow_mut() = Some(layer));
}

/// Moves a newly created service page into the layer and gives it `bounds`.
pub fn adopt<R: Runtime>(webview: &Webview<R>, bounds: ContentBounds) {
    let _ = webview.with_webview(move |platform| {
        let view: gtk::Widget = platform.inner().upcast();
        LAYER.with(|cell| {
            let Some(layer) = cell.borrow().clone() else { return };
            let in_layer =
                view.parent().is_some_and(|parent| parent == layer.clone().upcast::<gtk::Widget>());
            if !in_layer {
                if let Some(parent) = view.parent().and_then(|p| p.downcast::<gtk::Container>().ok()) {
                    parent.remove(&view);
                }
                layer.put(&view, 0, 0);
            }
            place(&layer, &view, bounds);
        });
    });
}

/// Gives a service page `bounds` within the window.
pub fn set_bounds<R: Runtime>(webview: &Webview<R>, bounds: ContentBounds) {
    let _ = webview.with_webview(move |platform| {
        let view: gtk::Widget = platform.inner().upcast();
        LAYER.with(|cell| {
            if let Some(layer) = cell.borrow().as_ref() {
                place(layer, &view, bounds);
            }
        });
    });
}

fn place(layer: &gtk::Fixed, view: &gtk::Widget, bounds: ContentBounds) {
    layer.set_margin_start(bounds.x.round() as i32);
    layer.set_margin_top(bounds.y.round() as i32);
    view.set_size_request(bounds.width.round().max(1.0) as i32, bounds.height.round().max(1.0) as i32);
}
