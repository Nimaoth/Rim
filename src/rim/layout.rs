use std::boxed::Box;

use super::view::View;

#[derive(Debug,Copy, Clone)]
pub enum LayoutDirection {
    Horizontal,
    Vertical,
}
pub trait Layout {
    fn layout(&self, views: &mut [View], width: i32, height: i32, direction: LayoutDirection);
    fn get_next_index(&self, view_count: usize, index: usize, x_off: i32, y_iff: i32, direction: LayoutDirection) -> usize;
}

pub struct GridLayout {}

impl GridLayout {
    pub fn new() -> Box<dyn Layout> {
        Box::new(GridLayout {})
    }
}

impl GridLayout {
    fn get_grid_size(&self, view_count: i32, direction: LayoutDirection) -> (i32, i32) {
        match direction {
            LayoutDirection::Horizontal => {
                let grid_columns = {
                    let cols = (view_count as f32).sqrt() as i32;
                    if cols * cols < view_count {
                        cols + 1
                    } else {
                        cols
                    }
                };
                let grid_rows = (view_count as f32 / grid_columns as f32).ceil() as i32;
                (grid_columns, grid_rows)
            },
            LayoutDirection::Vertical => {
                let grid_rows = {
                    let rows = (view_count as f32).sqrt() as i32;
                    if rows * rows < view_count {
                        rows + 1
                    } else {
                        rows
                    }
                };
                let grid_columns = (view_count as f32 / grid_rows as f32).ceil() as i32;
                (grid_columns, grid_rows)
            },
        }
    }
}

impl Layout for GridLayout {

    fn layout(&self, views: &mut [View], width: i32, height: i32, direction: LayoutDirection) {
        let (grid_columns, grid_rows) = self.get_grid_size(views.len() as i32, direction);

        let cell_width = width / grid_columns;
        let cell_height = height / grid_rows;

        for y in 0 .. grid_rows {
            for x in 0 .. grid_columns {
                let index = (x + y * grid_columns) as usize;
                if index >= views.len() {
                    continue;
                }
                let view = &mut views[index];
                view.x = x * cell_width;
                view.y = y * cell_height;
                view.width = cell_width;
                view.height = cell_height;
            }
        }
    }

    fn get_next_index(&self, view_count: usize, index: usize, x_off: i32, y_off: i32, direction: LayoutDirection) -> usize {
        let (grid_columns, grid_rows) = self.get_grid_size(view_count as i32, direction);
        let x = ((index as i32 % grid_columns + x_off) + grid_columns) % grid_columns;
        let y = ((index as i32 / grid_columns + y_off) + grid_rows) % grid_rows;

        let new_index = (x + y * grid_columns) as usize;
        if new_index >= view_count - 1 {
            return view_count - 1;
        }
        return new_index;
    }
}
