use std::boxed::Box;

use super::view::View;

pub trait Layout {
    fn layout(&self, views: &mut [View], width: i32, height: i32);
    fn get_next_index(&self, view_count: usize, index: usize, x_off: i32, y_iff: i32) -> usize;
}

pub struct GridLayout {}

impl GridLayout {
    pub fn new() -> Box<dyn Layout> {
        Box::new(GridLayout {})
    }
}

impl Layout for GridLayout {
    fn layout(&self, views: &mut [View], width: i32, height: i32) {
        let grid_columns = {
            let cols = (views.len() as f32).sqrt() as i32;
            if cols * cols < views.len() as i32 {
                cols + 1
            } else {
                cols
            }
        };
        let grid_rows = (views.len() as f32 / grid_columns as f32).ceil() as i32;

        let cell_width = width / grid_columns;
        let cell_height = height / grid_rows;

        // println!("cols: {}, rows: {}", grid_columns, grid_rows);
        for y in 0 .. grid_rows {
            for x in 0 .. grid_columns {
                let index = (x + y * grid_columns) as usize;
                if index >= views.len() {
                    break;
                }
                let view = &mut views[index];
                view.x = x * cell_width;
                view.y = y * cell_height;
                view.width = cell_width;
                view.height = cell_height;
            }
        }
    }

    fn get_next_index(&self, view_count: usize, index: usize, x_off: i32, y_off: i32) -> usize {
        let grid_columns = {
            let cols = (view_count as f32).sqrt() as i32;
            if cols * cols < view_count as i32 {
                cols + 1
            } else {
                cols
            }
        };
        let grid_rows = (view_count as f32 / grid_columns as f32).ceil() as i32;
        let x = ((index as i32 % grid_columns + x_off) + grid_columns) % grid_columns;
        let y = ((index as i32 / grid_columns + y_off) + grid_rows) % grid_columns;

        let new_index = (x + y * grid_columns) as usize;
        if new_index >= view_count - 1 {
            return view_count - 1;
        }
        return new_index;
    }
}
