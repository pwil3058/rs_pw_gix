// Copyright 2020 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use cairo::Operator;
use gdk::prelude::GdkContextExt;
use gdk_pixbuf;
use gtk::prelude::*;

use crate::geometry::Point;
use crate::gtkx::drawing_area::XYSelection;
use crate::{
    geometry::{AspectRatio, Rectangle, Size, SizeExt},
    gtkx::menu::{ManagedMenu, ManagedMenuBuilder},
    wrapper::*,
};

struct Zoomable {
    unzoomed: gdk_pixbuf::Pixbuf,
    zoomed: gdk_pixbuf::Pixbuf,
    zoom_factor: f64,
}

impl From<gdk_pixbuf::Pixbuf> for Zoomable {
    fn from(pixbuf: gdk_pixbuf::Pixbuf) -> Zoomable {
        Zoomable {
            unzoomed: pixbuf.clone(),
            zoomed: pixbuf.clone(),
            zoom_factor: 1.0,
        }
    }
}

impl AspectRatio for Zoomable {
    fn aspect_ratio(&self) -> f64 {
        self.unzoomed.aspect_ratio()
    }
}

impl Zoomable {
    pub fn pixbuf(&self) -> &gdk_pixbuf::Pixbuf {
        &self.zoomed
    }

    pub fn subpixbuf(&self, rect: Rectangle<i32>) -> Option<gdk_pixbuf::Pixbuf> {
        self.zoomed
            .new_subpixbuf(rect.x, rect.y, rect.width, rect.height)
    }

    pub fn zoom_factor(&self) -> f64 {
        self.zoom_factor
    }

    pub fn zoomed_size(&self) -> Size<f64> {
        self.zoomed.size().into()
    }

    pub fn set_zoom(&mut self, zoom_factor: f64) {
        let new_size = self.unzoomed.size() * zoom_factor;
        if let Some(new_pixbuf) = self.unzoomed.scale_simple(
            new_size.width,
            new_size.height,
            gdk_pixbuf::InterpType::Bilinear,
        ) {
            self.zoomed = new_pixbuf;
            self.zoom_factor = zoom_factor;
        } //TODO: do something about failure
    }

    pub fn set_zoomed_size(&mut self, new_zsize: Size<i32>) {
        assert!(self.aspect_ratio_matches_size(new_zsize.into()));
        if let Some(new_pixbuf) = self.unzoomed.scale_simple(
            new_zsize.width,
            new_zsize.height,
            gdk_pixbuf::InterpType::Bilinear,
        ) {
            self.zoomed = new_pixbuf;
            self.zoom_factor = self.zoomed.scale_versus(&self.unzoomed);
        } //TODO: do something about failure
    }

    pub fn calc_zooms_for(&self, size: Size<f64>) -> Size<f64> {
        size.scales_versus(self.unzoomed.size().into())
    }
}

#[derive(PWO, Wrapper)]
pub struct PixbufView {
    scrolled_window: gtk::ScrolledWindow,
    drawing_area: gtk::DrawingArea,
    xy_selection: Rc<XYSelection>,
    zoomable: RefCell<Option<Zoomable>>,
    selection_zoom: Cell<f64>,
    ignore_size_alloc: Cell<bool>,
    last_allocation: RefCell<Option<Size<f64>>>,
    zoom_in_adj: Cell<[f64; 2]>,
    zoom_out_adj: Cell<[f64; 2]>,
    last_xy: Cell<Point>,
    doing_button_motion: Cell<bool>,
    popup_menu: ManagedMenu,
}

impl PixbufView {
    const ZOOM_FACTOR: f64 = 1.005;
    const ZOOM_IN_ADJUST: f64 = (Self::ZOOM_FACTOR - 1.0) / 2.0;
    const ZOOM_OUT_ADJUST: f64 = (1.0 / Self::ZOOM_FACTOR - 1.0) / 2.0;

    pub fn set_pixbuf(&self, o_pixbuf: Option<&gdk_pixbuf::Pixbuf>) {
        if let Some(pixbuf) = o_pixbuf {
            self.xy_selection.reset();
            *self.zoomable.borrow_mut() = Some(Zoomable::from(pixbuf.clone()));
            if let Some(ref mut zoomable) = *self.zoomable.borrow_mut() {
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
        if let Some(ref mut zoomable) = *self.zoomable.borrow_mut() {
            let current_zoom = zoomable.zoom_factor();
            zoomable.set_zoom(current_zoom * Self::ZOOM_FACTOR);
            self.resize_drawing_area();
            for (dim, o_adj) in [
                self.scrolled_window.get_hadjustment(),
                self.scrolled_window.get_vadjustment(),
            ]
            .iter()
            .enumerate()
            {
                if let Some(ref adj) = *o_adj {
                    let new_val = adj.get_value() * Self::ZOOM_FACTOR + self.zoom_in_adj.get()[dim];
                    adj.set_value(new_val);
                }
            }
            self.selection_zoom.set(zoomable.zoom_factor())
        }
    }

    fn zoom_out(&self) {
        if let Some(ref mut zoomable) = *self.zoomable.borrow_mut() {
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
            zoomable.set_zoom(current_zoom / Self::ZOOM_FACTOR);
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
                        adj.get_value() / Self::ZOOM_FACTOR + self.zoom_out_adj.get()[dim];
                    adj.set_value(new_val.max(0.0));
                }
            }
            self.selection_zoom.set(zoomable.zoom_factor())
        }
    }
}

pub struct PixbufViewBuilder {}

impl PixbufViewBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn build(self) -> Rc<PixbufView> {
        let drawing_area = gtk::DrawingAreaBuilder::new().build();
        let xy_selection = XYSelection::create(&drawing_area);
        let scrolled_window = gtk::ScrolledWindowBuilder::new()
            .events(
                gdk::EventMask::POINTER_MOTION_MASK
                    | gdk::EventMask::BUTTON_PRESS_MASK
                    | gdk::EventMask::BUTTON_RELEASE_MASK
                    | gdk::EventMask::LEAVE_NOTIFY_MASK,
            )
            .child(&drawing_area)
            .build();

        let viewer = Rc::new(PixbufView {
            scrolled_window,
            drawing_area,
            xy_selection,
            zoomable: RefCell::new(None),
            ignore_size_alloc: Cell::new(false),
            selection_zoom: Cell::new(1.0),
            last_allocation: RefCell::new(None),
            zoom_in_adj: Cell::new([0.0, 0.0]),
            zoom_out_adj: Cell::new([0.0, 0.0]),
            last_xy: Cell::new(Point(0.0, 0.0)),
            doing_button_motion: Cell::new(false),
            popup_menu: ManagedMenuBuilder::new().build(),
        });

        let viewer_c = viewer.clone();
        viewer.drawing_area.connect_draw(move |_, cairo_context| {
            if let Some(ref zoomable) = *viewer_c.zoomable.borrow() {
                cairo_context.set_source_pixbuf(&zoomable.pixbuf(), 0.0, 0.0);
                cairo_context.paint();
                if viewer_c.xy_selection.is_drawable() {
                    let scale = zoomable.zoom_factor() / viewer_c.selection_zoom.get();
                    let rect = viewer_c.xy_selection.get_selected_rectangle(scale).unwrap();
                    if viewer_c.xy_selection.selection_made() {
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

        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_size_allocate(move |sw, allocation| {
                if viewer_c.ignore_size_alloc.get() {
                    return;
                };
                let alloc = Rectangle::<f64>::from(*allocation).size();
                let o_last_allocation = *viewer_c.last_allocation.borrow();
                if let Some(last_allocation) = o_last_allocation {
                    if last_allocation != alloc {
                        viewer_c
                            .zoom_in_adj
                            .set((alloc * PixbufView::ZOOM_IN_ADJUST).into());
                        viewer_c
                            .zoom_out_adj
                            .set((alloc * PixbufView::ZOOM_OUT_ADJUST).into());
                        *viewer_c.last_allocation.borrow_mut() = Some(alloc);
                        if let Some(ref mut zoomable) = *viewer_c.zoomable.borrow_mut() {
                            let delta_alloc = alloc - last_allocation;
                            let zoomed_sizediff = alloc - zoomable.zoomed_size();
                            if zoomable.aspect_ratio_matches_size(alloc)
                                && zoomed_sizediff.width.abs() < 10.0
                            {
                                // a small change and same aspect ratio
                                zoomable.set_zoomed_size(alloc.into());
                                viewer_c.resize_drawing_area();
                            } else if delta_alloc.width >= 0.0 {
                                if delta_alloc.height >= 0.0 {
                                    // we're getting bigger
                                    if zoomed_sizediff.width > 10.0 || zoomed_sizediff.height > 10.0
                                    {
                                        let zoom =
                                            zoomable.calc_zooms_for(alloc).length_longest_side();
                                        zoomable.set_zoom(zoom);
                                        viewer_c.resize_drawing_area();
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
                                    viewer_c.resize_drawing_area();
                                } else if zoomed_sizediff.width < -10.0
                                    && zoomed_sizediff.height < -10.0
                                {
                                    if zoomed_sizediff.width > -30.0
                                        || zoomed_sizediff.height > -30.0
                                    {
                                        let zoom =
                                            zoomable.calc_zooms_for(alloc).length_longest_side();
                                        zoomable.set_zoom(zoom);
                                        viewer_c.resize_drawing_area();
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
                    viewer_c
                        .zoom_in_adj
                        .set((alloc * PixbufView::ZOOM_IN_ADJUST).into());
                    viewer_c
                        .zoom_out_adj
                        .set((alloc * PixbufView::ZOOM_OUT_ADJUST).into());
                    *viewer_c.last_allocation.borrow_mut() = Some(alloc);
                }
            });

        // Set zoom using scroll wheel when control key pressed
        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_scroll_event(move |_, event| {
                if event.get_state().contains(gdk::ModifierType::CONTROL_MASK) {
                    match event.get_direction() {
                        gdk::ScrollDirection::Up => {
                            viewer_c.zoom_in();
                            return gtk::Inhibit(true);
                        }
                        gdk::ScrollDirection::Down => {
                            viewer_c.zoom_in();
                            return gtk::Inhibit(true);
                        }
                        gdk::ScrollDirection::Smooth => {
                            let (_, delta_y) = event.get_delta();
                            if delta_y > 0.0 {
                                viewer_c.zoom_in();
                                return gtk::Inhibit(true);
                            } else if delta_y < 0.0 {
                                viewer_c.zoom_out();
                                return gtk::Inhibit(true);
                            }
                        }
                        _ => (),
                    }
                };
                gtk::Inhibit(false)
            });

        let viewer_c = viewer.clone();
        viewer.xy_selection.connect_selection_made(move || {
            if let Some(ref zoomable) = *viewer_c.zoomable.borrow() {
                viewer_c.selection_zoom.set(zoomable.zoom_factor())
            } else {
                viewer_c.selection_zoom.set(1.0)
            };
            viewer_c.drawing_area.queue_draw()
        });

        // Set up moving image with left button and control key
        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_button_press_event(move |_, event| {
                if event.get_button() == 1
                    && event.get_state().contains(gdk::ModifierType::CONTROL_MASK)
                {
                    viewer_c.last_xy.set(event.get_position().into());
                    viewer_c.doing_button_motion.set(true);
                    return gtk::Inhibit(true);
                } else if event.get_button() == 3 {
                    viewer_c.popup_menu.popup_at_event(event);
                    return gtk::Inhibit(true);
                };
                gtk::Inhibit(false)
            });
        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_button_release_event(move |_, event| {
                if event.get_button() == 1 && viewer_c.doing_button_motion.get() {
                    viewer_c.doing_button_motion.set(false);
                    return gtk::Inhibit(true);
                };
                gtk::Inhibit(false)
            });
        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_leave_notify_event(move |_, _| {
                viewer_c.doing_button_motion.set(false);
                gtk::Inhibit(false)
            });
        let viewer_c = viewer.clone();
        viewer
            .scrolled_window
            .connect_motion_notify_event(move |_, event| {
                if viewer_c.doing_button_motion.get() {
                    let this_xy: Point = event.get_position().into();
                    let delta_xy: [f64; 2] = (this_xy - viewer_c.last_xy.get()).into();
                    viewer_c.last_xy.set(this_xy);
                    for (dim, o_adj) in [
                        viewer_c.scrolled_window.get_hadjustment(),
                        viewer_c.scrolled_window.get_vadjustment(),
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
        let viewer_c = Rc::clone(&viewer);
        viewer
            .popup_menu
            .append_item(
                "copy",
                "Copy",
                None,
                "Copy the selection to the clipboard",
                0,
            )
            .connect_activate(move |_| {
                if let Some(ref zoomable) = *viewer_c.zoomable.borrow() {
                    let scale = zoomable.zoom_factor() / viewer_c.selection_zoom.get();
                    if let Some(rect) = viewer_c.xy_selection.get_selected_rectangle(scale) {
                        if let Some(pixbuf) = zoomable.subpixbuf(rect.into()) {
                            let cbd = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
                            cbd.set_image(&pixbuf);
                        } else {
                            panic!("File: {:?} Line: {:?}", file!(), line!())
                        }
                    } else {
                        panic!("File: {:?} Line: {:?}", file!(), line!())
                    }
                } else {
                    panic!("File: {:?} Line: {:?}", file!(), line!())
                }
            });

        viewer
    }
}