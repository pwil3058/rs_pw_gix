// Copyright 2017 Peter Williams <pwil3058@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::{Cell, RefCell};
use std::convert::From;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use gdk;
use gdk::prelude::ContextExt;
use gdk_pixbuf::{self, PixbufExt};
use gtk;
use gtk::prelude::*;

use cairo::Operator;
use cairox::*;

use gtkx::drawing_area::*;
use recollections;
use wrapper::*;

struct Zoomable {
    unzoomed: gdk_pixbuf::Pixbuf,
    zoomed: RefCell<gdk_pixbuf::Pixbuf>,
    zoom_factor: Cell<f64>,
}

impl From<gdk_pixbuf::Pixbuf> for Zoomable {
    fn from(pixbuf: gdk_pixbuf::Pixbuf) -> Zoomable {
        Zoomable {
            unzoomed: pixbuf.clone(),
            zoomed: RefCell::new(pixbuf.clone()),
            zoom_factor: Cell::new(1.0),
        }
    }
}

impl AspectRatio for Zoomable {
    fn aspect_ratio(&self) -> f64 {
        self.unzoomed.aspect_ratio()
    }
}

impl Zoomable {
    pub fn get_pixbuf(&self) -> gdk_pixbuf::Pixbuf {
        self.zoomed.borrow().clone()
    }

    pub fn get_subpixbuf(&self, rect: Rectangle<i32>) -> gdk_pixbuf::Pixbuf {
        self.zoomed
            .borrow()
            .new_subpixbuf(rect.x, rect.y, rect.width, rect.height)
            .expect("Programmer Error")
    }

    pub fn zoom_factor(&self) -> f64 {
        self.zoom_factor.get()
    }

    pub fn zoomed_size(&self) -> Size<f64> {
        self.zoomed.borrow().size().into()
    }

    pub fn set_zoom(&self, zoom_factor: f64) {
        let new_size = self.unzoomed.size() * zoom_factor;
        if let Some(new_pixbuf) = self.unzoomed.scale_simple(
            new_size.width,
            new_size.height,
            gdk_pixbuf::InterpType::Bilinear,
        ) {
            *self.zoomed.borrow_mut() = new_pixbuf;
            self.zoom_factor.set(zoom_factor);
        } //TODO: do something about failure
    }

    pub fn set_zoomed_size(&self, new_zsize: Size<i32>) {
        assert!(self.aspect_ratio_matches_size(new_zsize.into()));
        if let Some(new_pixbuf) = self.unzoomed.scale_simple(
            new_zsize.width,
            new_zsize.height,
            gdk_pixbuf::InterpType::Bilinear,
        ) {
            *self.zoomed.borrow_mut() = new_pixbuf;
            self.zoom_factor
                .set(self.zoomed.borrow().scale_versus(&self.unzoomed));
        } //TODO: do something about failure
    }

    pub fn calc_zooms_for(&self, size: Size<f64>) -> Size<f64> {
        size.scales_versus(self.unzoomed.size().into())
    }
}

pub struct PixbufViewCore {
    scrolled_window: gtk::ScrolledWindow,
    drawing_area: gtk::DrawingArea,
    menu: gtk::Menu,
    copy_selection_item: gtk::MenuItem,
    load_image_item: gtk::MenuItem,
    print_image_item: gtk::MenuItem,
    xy_selection: XYSelection,
    last_allocation: RefCell<Option<Size<f64>>>,
    zoomable: RefCell<Option<Zoomable>>,
    selection_zoom: Cell<f64>,
    ignore_size_alloc: Cell<bool>,
    doing_button_motion: Cell<bool>,
    last_xy: Cell<Point>,
    zoom_in_adj: Cell<[f64; 2]>,
    zoom_out_adj: Cell<[f64; 2]>,
    current_file_path: RefCell<Option<PathBuf>>,
}

impl_widget_wrapper!(scrolled_window: gtk::ScrolledWindow, PixbufViewCore);

pub type PixbufView = Rc<PixbufViewCore>;

pub trait PixbufViewInterface {
    fn create() -> PixbufView;
}

impl PixbufViewCore {
    fn zoom_factor() -> f64 {
        1.005
    }
    fn zoom_in_adjust(&self) -> f64 {
        (Self::zoom_factor() - 1.0) / 2.0
    }
    fn zoom_out_adjust(&self) -> f64 {
        (1.0 / Self::zoom_factor() - 1.0) / 2.0
    }

    pub fn pwo(&self) -> gtk::ScrolledWindow {
        self.scrolled_window.clone()
    }

    pub fn set_pixbuf(&self, o_pixbuf: Option<&gdk_pixbuf::Pixbuf>) {
        if let Some(pixbuf) = o_pixbuf {
            self.xy_selection.reset();
            *self.zoomable.borrow_mut() = Some(Zoomable::from(pixbuf.clone()));
            if let Some(ref zoomable) = *self.zoomable.borrow() {
                let alloc = self.drawing_area.get_allocation().size();
                if pixbuf.aspect_ratio_matches_size(alloc.into()) {
                    zoomable.set_zoomed_size(alloc);
                } else {
                    let zoom = zoomable.calc_zooms_for(alloc.into()).length_longest_side();
                    zoomable.set_zoom(zoom);
                }
                self.resize_drawing_area();
            } else {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            }
        } else {
            *self.zoomable.borrow_mut() = None
        };
        self.drawing_area.queue_draw();
    }

    pub fn set_pixbuf_fm_file<P: AsRef<Path>>(
        &self,
        file_path: P,
    ) -> Result<(), gdk_pixbuf::Error> {
        let pixbuf = gdk_pixbuf::Pixbuf::new_from_file(file_path.as_ref())?;
        self.set_pixbuf(Some(&pixbuf));
        *self.current_file_path.borrow_mut() = Some(file_path.as_ref().to_path_buf());
        if let Some(current_file_path) = self.current_file_path() {
            let o_path_str = current_file_path.to_str();
            if let Some(path_str) = o_path_str {
                recollections::remember("image_viewer::last_image_file", path_str);
            }
        }
        Ok(())
    }

    pub fn reload_last_image(&self) {
        let o_last_file = recollections::recall("image_viewer::last_image_file");
        if let Some(ref last_file_path) = o_last_file {
            if let Err(err) = self.set_pixbuf_fm_file(last_file_path) {
                self.inform_user("Failed To Load Previous Image", Some(err.description()));
            };
        };
    }

    pub fn current_file_path(&self) -> Option<PathBuf> {
        match *self.current_file_path.borrow() {
            Some(ref path_buf) => Some(path_buf.clone()),
            None => None,
        }
    }

    fn resize_drawing_area(&self) {
        if let Some(ref zoomable) = *self.zoomable.borrow() {
            self.ignore_size_alloc.set(true);
            let new_size: Size<i32> = zoomable.zoomed_size().into();
            self.drawing_area
                .set_size_request(new_size.width, new_size.height);
            let sizediff = self.scrolled_window.get_allocation().size() - new_size;
            if sizediff.width >= 0 && sizediff.height >= 0 {
                self.scrolled_window
                    .set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never)
            } else {
                self.scrolled_window
                    .set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic)
            };
            self.ignore_size_alloc.set(false);
            self.selection_zoom.set(zoomable.zoom_factor())
        }
    }

    fn zoom_in(&self) {
        if let Some(ref zoomable) = *self.zoomable.borrow() {
            let current_zoom = zoomable.zoom_factor();
            zoomable.set_zoom(current_zoom * Self::zoom_factor());
            self.resize_drawing_area();
            for (dim, o_adj) in [
                self.scrolled_window.get_hadjustment(),
                self.scrolled_window.get_vadjustment(),
            ]
            .iter()
            .enumerate()
            {
                if let Some(ref adj) = *o_adj {
                    let new_val =
                        adj.get_value() * Self::zoom_factor() + self.zoom_in_adj.get()[dim];
                    adj.set_value(new_val);
                }
            }
            self.selection_zoom.set(zoomable.zoom_factor())
        }
    }

    fn zoom_out(&self) {
        if let Some(ref zoomable) = *self.zoomable.borrow() {
            let current_zoom = zoomable.zoom_factor();
            let min_zoom = if let Some(alloc) = *self.last_allocation.borrow() {
                zoomable.calc_zooms_for(alloc).length_longest_side()
            } else {
                1.0
            };
            if current_zoom <= min_zoom {
                gdk::beep();
                return;
            };
            zoomable.set_zoom(current_zoom / Self::zoom_factor());
            self.resize_drawing_area();
            for (dim, o_adj) in [
                self.scrolled_window.get_hadjustment(),
                self.scrolled_window.get_vadjustment(),
            ]
            .iter()
            .enumerate()
            {
                if let Some(ref adj) = *o_adj {
                    let new_val =
                        adj.get_value() / Self::zoom_factor() + self.zoom_out_adj.get()[dim];
                    adj.set_value(new_val.max(0.0));
                }
            }
            self.selection_zoom.set(zoomable.zoom_factor())
        }
    }
}

impl PixbufViewInterface for PixbufView {
    fn create() -> PixbufView {
        let scrolled_window = gtk::ScrolledWindow::new(None, None);
        scrolled_window.set_policy(gtk::PolicyType::Automatic, gtk::PolicyType::Automatic);
        let events = gdk::EventMask::POINTER_MOTION_MASK
            | gdk::EventMask::BUTTON_PRESS_MASK
            | gdk::EventMask::BUTTON_RELEASE_MASK
            | gdk::EventMask::LEAVE_NOTIFY_MASK;
        scrolled_window.add_events(events.bits() as i32);
        let drawing_area = gtk::DrawingArea::new();
        scrolled_window.add(&drawing_area);
        let xy_selection = XYSelection::create(&drawing_area);

        let menu = gtk::Menu::new();
        let copy_selection_item = gtk::MenuItem::new_with_label("Copy");
        copy_selection_item.set_tooltip_text("Copy the selection to the clipboard");
        menu.append(&copy_selection_item.clone());
        let load_image_item = gtk::MenuItem::new_with_label("Load");
        load_image_item.set_tooltip_text("Load an image from a file");
        menu.append(&load_image_item.clone());
        let print_image_item = gtk::MenuItem::new_with_label("Print");
        print_image_item.set_tooltip_text("Print the image");
        menu.append(&print_image_item.clone());
        menu.show_all();

        let pbv = Rc::new(PixbufViewCore {
            scrolled_window: scrolled_window,
            drawing_area: drawing_area,
            menu: menu,
            copy_selection_item: copy_selection_item,
            load_image_item: load_image_item,
            print_image_item: print_image_item,
            xy_selection: xy_selection,
            last_allocation: RefCell::new(None),
            zoomable: RefCell::new(None),
            selection_zoom: Cell::new(1.0),
            ignore_size_alloc: Cell::new(false),
            doing_button_motion: Cell::new(false),
            last_xy: Cell::new(Point(0.0, 0.0)),
            zoom_in_adj: Cell::new([0.0, 0.0]),
            zoom_out_adj: Cell::new([0.0, 0.0]),
            current_file_path: RefCell::new(None),
        });
        let pbv_c = pbv.clone();
        pbv.drawing_area.connect_draw(move |_, cairo_context| {
            if let Some(ref zoomable) = *pbv_c.zoomable.borrow() {
                cairo_context.set_source_pixbuf(&zoomable.get_pixbuf(), 0.0, 0.0);
                cairo_context.paint();
                if pbv_c.xy_selection.is_drawable() {
                    let scale = zoomable.zoom_factor() / pbv_c.selection_zoom.get();
                    let rect = pbv_c.xy_selection.get_selected_rectangle(scale).unwrap();
                    if pbv_c.xy_selection.selection_made() {
                        cairo_context.set_dash(&[], 0.0)
                    } else {
                        cairo_context.set_dash(&[3.0], 0.0)
                    };
                    cairo_context.rectangle(rect.x, rect.y, rect.width, rect.height);
                    cairo_context.set_source_rgb(0.0, 0.0, 0.0);
                    cairo_context.set_operator(Operator::Xor);
                    cairo_context.stroke();
                }
            };
            gtk::Inhibit(false)
        });
        let pbv_c = pbv.clone();
        pbv.scrolled_window
            .connect_size_allocate(move |sw, allocation| {
                if pbv_c.ignore_size_alloc.get() {
                    return;
                };
                let alloc = Rectangle::<f64>::from(*allocation).size();
                let o_last_allocation = *pbv_c.last_allocation.borrow();
                if let Some(last_allocation) = o_last_allocation {
                    if last_allocation != alloc {
                        pbv_c
                            .zoom_in_adj
                            .set((alloc * pbv_c.zoom_in_adjust()).into());
                        pbv_c
                            .zoom_out_adj
                            .set((alloc * pbv_c.zoom_out_adjust()).into());
                        *pbv_c.last_allocation.borrow_mut() = Some(alloc);
                        if let Some(ref zoomable) = *pbv_c.zoomable.borrow() {
                            let delta_alloc = alloc - last_allocation;
                            let zoomed_sizediff = alloc - zoomable.zoomed_size();
                            if zoomable.aspect_ratio_matches_size(alloc)
                                && zoomed_sizediff.width.abs() < 10.0
                            {
                                // a small change and same aspect ratio
                                zoomable.set_zoomed_size(alloc.into());
                                pbv_c.resize_drawing_area();
                            } else if delta_alloc.width >= 0.0 {
                                if delta_alloc.height >= 0.0 {
                                    // we're getting bigger
                                    if zoomed_sizediff.width > 10.0 || zoomed_sizediff.height > 10.0
                                    {
                                        let zoom =
                                            zoomable.calc_zooms_for(alloc).length_longest_side();
                                        zoomable.set_zoom(zoom);
                                        pbv_c.resize_drawing_area();
                                    } else if zoomed_sizediff.width < 0.0
                                        || zoomed_sizediff.height < 0.0
                                    {
                                        sw.set_policy(
                                            gtk::PolicyType::Automatic,
                                            gtk::PolicyType::Automatic,
                                        )
                                    } else {
                                        sw.set_policy(
                                            gtk::PolicyType::Never,
                                            gtk::PolicyType::Never,
                                        )
                                    }
                                } else {
                                    // uncharted territory
                                }
                            } else if delta_alloc.height <= 0.0 {
                                // we're getting smaller
                                if zoomed_sizediff.width > 10.0 || zoomed_sizediff.height > 10.0 {
                                    let zoom = zoomable.calc_zooms_for(alloc).length_longest_side();
                                    zoomable.set_zoom(zoom);
                                    pbv_c.resize_drawing_area();
                                } else if zoomed_sizediff.width < -10.0
                                    && zoomed_sizediff.height < -10.0
                                {
                                    if zoomed_sizediff.width > -30.0
                                        || zoomed_sizediff.height > -30.0
                                    {
                                        let zoom =
                                            zoomable.calc_zooms_for(alloc).length_longest_side();
                                        zoomable.set_zoom(zoom);
                                        pbv_c.resize_drawing_area();
                                    }
                                } else if zoomed_sizediff.width < 0.0
                                    || zoomed_sizediff.height < 0.0
                                {
                                    sw.set_policy(
                                        gtk::PolicyType::Automatic,
                                        gtk::PolicyType::Automatic,
                                    )
                                } else {
                                    sw.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Never)
                                }
                            } else {
                                // more uncharted territory
                            }
                        }
                    }
                } else {
                    pbv_c
                        .zoom_in_adj
                        .set((alloc * pbv_c.zoom_in_adjust()).into());
                    pbv_c
                        .zoom_out_adj
                        .set((alloc * pbv_c.zoom_out_adjust()).into());
                    *pbv_c.last_allocation.borrow_mut() = Some(alloc);
                }
            });
        // Set zoom using scroll wheel when control key pressed
        let pbv_c = pbv.clone();
        pbv.scrolled_window.connect_scroll_event(move |_, event| {
            if event.get_state().contains(gdk::ModifierType::CONTROL_MASK) {
                match event.get_direction() {
                    gdk::ScrollDirection::Up => {
                        pbv_c.zoom_in();
                        return gtk::Inhibit(true);
                    }
                    gdk::ScrollDirection::Down => {
                        pbv_c.zoom_in();
                        return gtk::Inhibit(true);
                    }
                    gdk::ScrollDirection::Smooth => {
                        let (_, delta_y) = event.get_delta();
                        if delta_y > 0.0 {
                            pbv_c.zoom_in();
                            return gtk::Inhibit(true);
                        } else if delta_y < 0.0 {
                            pbv_c.zoom_out();
                            return gtk::Inhibit(true);
                        }
                    }
                    _ => (),
                }
            };
            gtk::Inhibit(false)
        });
        let pbv_c = pbv.clone();
        pbv.xy_selection.connect_selection_made(move || {
            if let Some(ref zoomable) = *pbv_c.zoomable.borrow() {
                pbv_c.selection_zoom.set(zoomable.zoom_factor())
            } else {
                pbv_c.selection_zoom.set(1.0)
            };
            pbv_c.drawing_area.queue_draw()
        });
        // Set up moving image with left button and control key
        let pbv_c = pbv.clone();
        pbv.scrolled_window
            .connect_button_press_event(move |_, event| {
                if event.get_button() == 1
                    && event.get_state().contains(gdk::ModifierType::CONTROL_MASK)
                {
                    pbv_c.last_xy.set(event.get_position().into());
                    pbv_c.doing_button_motion.set(true);
                    return gtk::Inhibit(true);
                } else if event.get_button() == 3 {
                    if pbv_c.zoomable.borrow().is_some() {
                        pbv_c.print_image_item.set_sensitive(true);
                        pbv_c
                            .copy_selection_item
                            .set_sensitive(pbv_c.xy_selection.selection_made());
                    } else {
                        pbv_c.print_image_item.set_sensitive(false);
                        pbv_c.copy_selection_item.set_sensitive(false);
                    };
                    // TODO: needs v3_22: pbv_c.menu.popup_at_pointer(None);
                    pbv_c.menu.popup_easy(event.get_button(), event.get_time());
                    return gtk::Inhibit(true);
                };
                gtk::Inhibit(false)
            });
        let pbv_c = pbv.clone();
        pbv.scrolled_window
            .connect_button_release_event(move |_, event| {
                if event.get_button() == 1 && pbv_c.doing_button_motion.get() {
                    pbv_c.doing_button_motion.set(false);
                    return gtk::Inhibit(true);
                };
                gtk::Inhibit(false)
            });
        let pbv_c = pbv.clone();
        pbv.scrolled_window.connect_leave_notify_event(move |_, _| {
            pbv_c.doing_button_motion.set(false);
            gtk::Inhibit(false)
        });
        let pbv_c = pbv.clone();
        pbv.scrolled_window
            .connect_motion_notify_event(move |_, event| {
                if pbv_c.doing_button_motion.get() {
                    let this_xy: Point = event.get_position().into();
                    let delta_xy: [f64; 2] = (this_xy - pbv_c.last_xy.get()).into();
                    pbv_c.last_xy.set(this_xy);
                    for (dim, o_adj) in [
                        pbv_c.scrolled_window.get_hadjustment(),
                        pbv_c.scrolled_window.get_vadjustment(),
                    ]
                    .iter()
                    .enumerate()
                    {
                        if let Some(ref adj) = *o_adj {
                            let new_val = adj.get_value() - delta_xy[dim];
                            adj.set_value(
                                new_val
                                    .max(adj.get_lower())
                                    .min(adj.get_upper() - adj.get_page_size()),
                            );
                        }
                    }
                    return gtk::Inhibit(true);
                };
                gtk::Inhibit(false)
            });
        // POPUP MENU
        let pbv_c = pbv.clone();
        pbv.copy_selection_item.clone().connect_activate(move |_| {
            if let Some(ref zoomable) = *pbv_c.zoomable.borrow() {
                let scale = zoomable.zoom_factor() / pbv_c.selection_zoom.get();
                if let Some(rect) = pbv_c.xy_selection.get_selected_rectangle(scale) {
                    let pixbuf = zoomable.get_subpixbuf(rect.into());
                    let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                    cbd.set_image(&pixbuf);
                } else {
                    panic!("File: {:?} Line: {:?}", file!(), line!())
                }
            } else {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            }
        });
        let pbv_c = pbv.clone();
        pbv.load_image_item.clone().connect_activate(move |_| {
            let o_last_file = recollections::recall("image_viewer::last_image_file");
            let last_file = if let Some(ref text) = o_last_file {
                Some(text.as_str())
            } else {
                None
            };
            if let Some(path) = pbv_c.ask_file_path(Some("Image File"), last_file, true) {
                if let Err(err) = pbv_c.set_pixbuf_fm_file(path) {
                    pbv_c.inform_user("Failed To Load Image", Some(err.description()));
                }
            }
        });
        let pbv_c = pbv.clone();
        pbv.print_image_item.clone().connect_activate(move |_| {
            if let Some(ref zoomable) = *pbv_c.zoomable.borrow() {
                if let Err(ref err) = pbv_c.print_pixbuf(&zoomable.get_pixbuf()) {
                    pbv_c.report_error("Print Error", err);
                }
            } else {
                panic!("File: {:?} Line: {:?}", file!(), line!())
            }
        });
        pbv.scrolled_window.show_all();

        pbv
    }
}

#[cfg(test)]
mod tests {
    //use super::*;

    #[test]
    fn it_works() {}
}
