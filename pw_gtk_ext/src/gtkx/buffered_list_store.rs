// Copyright 2021 Peter Williams <pwil3058@gmail.com> <pwil3058@bigpond.net.au>

use crate::glib::Value;
pub use crate::gtkx::list_store::*;
use std::cell::RefCell;
use std::rc::Rc;

pub trait RowDataSource: ListViewSpec + Sized {
    fn generate_rows(&self) -> Vec<Vec<Value>>;
    fn refresh(&self) -> Vec<u8>;
}

pub struct RowBufferCore<R: RowDataSource> {
    pub row_data_source: R,
    pub row_data_source_digest: Vec<u8>,
    pub rows: Rc<Vec<Vec<Value>>>,
    pub rows_digest: Vec<u8>,
}

pub struct RowBuffer<R: RowDataSource>(RefCell<RowBufferCore<R>>);

impl<R: RowDataSource> RowBuffer<R> {
    pub fn new(raw_data: R) -> Self {
        let rwc = RowBufferCore {
            row_data_source: raw_data,
            row_data_source_digest: vec![],
            rows: Rc::new(vec![]),
            rows_digest: vec![],
        };
        let row_buffer = Self(RefCell::new(rwc));
        row_buffer.init();
        row_buffer
    }

    pub fn columns() -> Vec<gtk::TreeViewColumn> {
        R::columns()
    }

    fn finalise(&self) {
        let mut core = self.0.borrow_mut();
        core.rows = Rc::new(core.row_data_source.generate_rows());
        core.rows_digest = core.row_data_source_digest.clone();
    }

    fn get_rows(&self) -> Rc<Vec<Vec<glib::Value>>> {
        let core = self.0.borrow();
        Rc::clone(&core.rows)
    }

    fn init(&self) {
        {
            let mut core = self.0.borrow_mut();
            core.row_data_source_digest = core.row_data_source.refresh();
        }
        self.finalise();
    }

    fn is_current(&self) -> bool {
        let mut core = self.0.borrow_mut();
        core.row_data_source_digest = core.row_data_source.refresh();
        core.row_data_source_digest == core.rows_digest
    }
}

pub struct BufferedListStore<R: RowDataSource> {
    list_store: gtk::ListStore,
    row_buffer: RowBuffer<R>,
}

impl<R: RowDataSource> BufferedListStore<R> {
    pub fn new(raw_data_source: R) -> Self {
        let list_store = gtk::ListStore::new(&R::column_types());
        let row_buffer = RowBuffer::new(raw_data_source);
        Self {
            list_store,
            row_buffer,
        }
    }

    pub fn repopulate(&self) {
        self.row_buffer.init();
        self.list_store.repopulate_with(&self.row_buffer.get_rows());
    }

    pub fn update(&self) {
        if !self.row_buffer.is_current() {
            // this does a raw data update
            self.row_buffer.finalise();
            self.list_store.update_with(&self.row_buffer.get_rows());
        };
    }
}

impl<R: RowDataSource> WrappedTreeModel<gtk::ListStore> for BufferedListStore<R> {
    fn columns() -> Vec<gtk::TreeViewColumn> {
        R::columns()
    }

    fn model(&self) -> &gtk::ListStore {
        &self.list_store
    }
}
