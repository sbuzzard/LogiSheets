#[macro_use]
extern crate logiutils;
#[macro_use]
extern crate lazy_static;
extern crate logisheets_math;
extern crate num;
extern crate rand;
extern crate regex;
extern crate statrs;
extern crate unicode_segmentation;

mod addr_parser;
mod async_func_manager;
mod calc_engine;
mod cell;
mod cell_attachments;
mod connectors;
mod container;
pub mod controller;
mod data_executor;
mod ext_book_manager;
mod file_loader2;
mod id_manager;
mod navigator;
mod payloads;
mod settings;
mod style_manager;
mod utils;
mod vertex_manager;
mod workbook;

use connectors::NameFetcher;
use controller::display::Comment;
pub use controller::{
    display::{MergeCell, Style, Value},
    Controller,
};
use logisheets_workbook::prelude::SerdeErr;
use parser::unparse;

pub type SheetId = controller_base::SheetId;
pub type CellId = controller_base::CellId;
pub type AsyncCalcResult = controller_base::async_func::AsyncCalcResult;
pub type AsyncErr = controller_base::async_func::AsyncErr;
pub type Task = controller_base::async_func::Task;
pub type BlockId = controller_base::BlockId;

// Has SKIPPED the '='
pub fn lex_success(f: &str) -> bool {
    let toks = lexer::lex(f);
    match toks {
        Some(_) => true,
        None => false,
    }
}

pub enum Err {
    SerdeErr(SerdeErr),
    NotFound,
}

pub struct Workbook {
    controller: Controller,
}

impl Workbook {
    /// ```
    /// use std::fs;
    /// let buf = fs::read("./example.xlsx");
    /// Workbook::from_file(&buf, String::from("example.xlsx"));
    /// ```
    pub fn from_file(buf: &[u8], book_name: String) -> Result<Self, Err> {
        match Controller::from_file(book_name, buf) {
            Ok(controller) => Ok(Workbook { controller }),
            Err(e) => Err(Err::SerdeErr(e)),
        }
    }

    /// ```
    /// use std::fs;
    /// let buf = fs::read("./example.xlsx");
    /// let mut workbook = Workbook::from_file(&buf, String::from("example.xlsx")).unwrap();
    /// workbook.get_sheet_by_name("Sheet1");
    /// ```
    pub fn get_sheet_by_name(&mut self, name: &str) -> Result<Worksheet, Err> {
        match self.controller.get_sheet_id_by_name(name) {
            Some(sheet_id) => Ok(Worksheet {
                sheet_id,
                controller: &mut self.controller,
            }),
            None => Err(Err::NotFound),
        }
    }

    /// ```
    /// use std::fs;
    /// let buf = fs::read("./example.xlsx");
    /// let mut workbook = Workbook::from_file(&buf, String::from("example.xlsx")).unwrap();
    /// workbook.get_sheet_by_idx(0);
    /// ```
    pub fn get_sheet_by_idx(&mut self, idx: usize) -> Result<Worksheet, Err> {
        match self.controller.get_sheet_id_by_idx(idx) {
            Some(sheet_id) => Ok(Worksheet {
                sheet_id,
                controller: &mut self.controller,
            }),
            None => Err(Err::NotFound),
        }
    }
}

pub struct Worksheet<'a> {
    sheet_id: SheetId,
    controller: &'a mut Controller,
}

impl<'a> Worksheet<'a> {
    pub fn get_value(&mut self, row: usize, col: usize) -> Result<Value, Err> {
        if let Some(cell_id) =
            self.controller
                .status
                .navigator
                .fetch_cell_id(self.sheet_id, row, col)
        {
            if let Some(cell) = self
                .controller
                .status
                .container
                .get_cell(self.sheet_id, &cell_id)
            {
                let value = &cell.value;
                let v = match value {
                    controller_base::CellValue::Blank => Value::Empty,
                    controller_base::CellValue::Boolean(b) => Value::Bool(*b),
                    controller_base::CellValue::Date(_) => Value::Empty,
                    controller_base::CellValue::Error(e) => Value::Error(e.to_string()),
                    controller_base::CellValue::String(s) => {
                        let text = self.controller.status.text_id_manager.get_string(s);
                        match text {
                            Some(r) => Value::Str(r),
                            None => Value::Str(String::from("")),
                        }
                    }
                    controller_base::CellValue::Number(n) => Value::Number(*n),
                    controller_base::CellValue::InlineStr(_) => Value::Empty,
                    controller_base::CellValue::FormulaStr(s) => Value::Str(s.to_string()),
                };
                Ok(v)
            } else {
                Ok(Value::Empty)
            }
        } else {
            Err(Err::NotFound)
        }
    }

    pub fn get_formula(&mut self, row: usize, col: usize) -> Result<String, Err> {
        if let Some(cell_id) =
            self.controller
                .status
                .navigator
                .fetch_cell_id(self.sheet_id, row, col)
        {
            if let Some(node) = self
                .controller
                .status
                .vertex_manager
                .status
                .formulas
                .get(&(self.sheet_id, cell_id))
            {
                let mut name_fetcher = NameFetcher {
                    func_manager: &mut self.controller.status.func_id_manager,
                    sheet_id_manager: &mut self.controller.status.sheet_id_manager,
                    external_links_manager: &mut self.controller.status.external_links_manager,
                    text_id_manager: &mut self.controller.status.text_id_manager,
                    name_id_manager: &mut self.controller.status.name_id_manager,
                    navigator: &mut self.controller.status.navigator,
                };
                let f = unparse::unparse(node, &mut name_fetcher, self.sheet_id);
                Ok(f)
            } else {
                Ok(String::from(""))
            }
        } else {
            Err(Err::NotFound)
        }
    }

    pub fn get_style(&mut self, row: usize, col: usize) -> Result<Style, Err> {
        if let Some(cell_id) =
            self.controller
                .status
                .navigator
                .fetch_cell_id(self.sheet_id, row, col)
        {
            let style_id = if let Some(cell) = self
                .controller
                .status
                .container
                .get_cell(self.sheet_id, &cell_id)
            {
                cell.style
            } else {
                let row_id = self
                    .controller
                    .status
                    .navigator
                    .fetch_row_id(self.sheet_id, row)
                    .unwrap();
                if let Some(row_info) = self
                    .controller
                    .status
                    .container
                    .get_row_info(self.sheet_id, row_id)
                {
                    row_info.style
                } else {
                    let col_id = self
                        .controller
                        .status
                        .navigator
                        .fetch_row_id(self.sheet_id, col)
                        .unwrap();
                    match self
                        .controller
                        .status
                        .container
                        .get_col_info(self.sheet_id, col_id)
                    {
                        Some(col_info) => col_info.style,
                        None => 0,
                    }
                }
            };
            let style = self
                .controller
                .status
                .style_manager
                .get_cell_style(style_id);
            Ok(style)
        } else {
            Err(Err::NotFound)
        }
    }

    pub fn get_merge_cells(&mut self) -> Vec<MergeCell> {
        let merges = self
            .controller
            .status
            .cell_attachment_manager
            .merge_cells
            .data
            .get(&self.sheet_id);
        if let Some(merges) = merges {
            merges
                .clone()
                .iter()
                .fold(vec![], |mut prev, (start, end)| {
                    let s = self
                        .controller
                        .status
                        .navigator
                        .fetch_normal_cell_idx(self.sheet_id, start);
                    let e = self
                        .controller
                        .status
                        .navigator
                        .fetch_normal_cell_idx(self.sheet_id, end);
                    match (s, e) {
                        (Some((row_start, col_start)), Some((row_end, col_end))) => {
                            let m = MergeCell {
                                row_start,
                                col_start,
                                row_end,
                                col_end,
                            };
                            prev.push(m);
                            prev
                        }
                        _ => prev,
                    }
                })
        } else {
            vec![]
        }
    }

    pub fn get_comments(&mut self) -> Vec<Comment> {
        let comments = self
            .controller
            .status
            .cell_attachment_manager
            .comments
            .data
            .get(&self.sheet_id);
        if let Some(comments) = comments {
            comments
                .comments
                .clone()
                .iter()
                .fold(vec![], |mut prev, (id, c)| {
                    if let Some((row, col)) = self
                        .controller
                        .status
                        .navigator
                        .fetch_cell_idx(self.sheet_id, id)
                    {
                        let author_id = c.author;
                        let author = self
                            .controller
                            .status
                            .cell_attachment_manager
                            .comments
                            .get_author_name(&author_id)
                            .unwrap();
                        let content = c.text.to_string();
                        let comment = Comment {
                            row,
                            col,
                            author,
                            content,
                        };
                        prev.push(comment);
                        prev
                    } else {
                        prev
                    }
                })
        } else {
            vec![]
        }
    }

    /// Get the dimension of the sheet.
    pub fn get_sheet_dimension(&mut self) -> (usize, usize) {
        let sheet_container = self
            .controller
            .status
            .container
            .get_sheet_container(self.sheet_id);
        sheet_container
            .cells
            .clone()
            .iter()
            .fold((0, 0), |(r, c), (id, _)| {
                let cell_idx = self
                    .controller
                    .status
                    .navigator
                    .fetch_cell_idx(self.sheet_id, id);
                match cell_idx {
                    Some((row, col)) => {
                        let r = if r > row { r } else { row };
                        let c = if c > col { c } else { col };
                        (r, c)
                    }
                    None => (r, c),
                }
            })
    }
}
