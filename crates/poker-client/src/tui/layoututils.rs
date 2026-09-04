// This file contains some functions that help with layout
// Note that virtually NO ERROR CHECKING takes place
// We assume that this is done BEFORE passing rects to this function

use ratatui::layout::Rect;

/// Gets the inner rect by symmetrical padding. You can use this to create
/// rects where you can put text, mostly, probably.
/// Pass it the outer rect, and how much
/// you would like to cut into the outer rect.
/// Using zero for these values, will simply return the original rect
/// Two is probably what you want for texts
pub fn get_inner_rect_pad(outer_rect: Rect, x: u16, y: u16) -> Rect {
    get_inner_rect(outer_rect, x, x, y, y)
}

/// Gets the inner rect.
/// Pass it the outer rect, and how much
/// you would like to cut into the outer rect.
/// Pass it the number of cells you want to remove from
///   left, right, top , bottom
/// Using zero for these values, will simply return the original rect
pub fn get_inner_rect(outer_rect: Rect, left: u16, right: u16, top: u16, bottom: u16) -> Rect {
    Rect {
        x: outer_rect.x + left,
        y: outer_rect.y + top,
        width: outer_rect.width - left - right,
        height: outer_rect.height - top - bottom,
    }
}

/// Function that creates a centered rect in the parent rect
/// If the requested size is bigger than the parent rect,
///   this function returns the parent rect.
///
/// So please check the sizes yourself
///
/// Pass it the required width and height of the centered rect
///   and the parent rect (typically the frame.area())
pub fn get_center_rect(parent: Rect, width: u16, height: u16) -> Rect {
    // Edge case, the requested size is bigger than the parent
    // simply return the parent
    if width >= parent.width || height >= parent.height {
        return parent;
    }

    let half_width = width / 2;
    let half_height = height / 2;

    Rect {
        x: parent.x + (parent.width / 2 - half_width),
        y: parent.y + (parent.height / 2 - half_height),
        width,
        height,
    }
}

/// Divide the screen in two rects
/// Pass it the number of cells the LEFT rect should be
pub fn horizontal_left_divide_by_cells(outer_rect: Rect, cells: u16) -> (Rect, Rect) {
    let left_rect = Rect {
        x: outer_rect.x,
        y: outer_rect.y,
        width: cells,
        height: outer_rect.height,
    };

    let right_rect = Rect {
        x: outer_rect.x + cells,
        y: outer_rect.y,
        width: outer_rect.width - cells,
        height: outer_rect.height,
    };

    (left_rect, right_rect)
}

/// Divide the screen in two rects
/// Pass it the by how much to divide the LEFT part
/// Eg. 3 means the left part is 1/3 or the outer rect, etc.
pub fn horizontal_left_divide_by_fraction(outer_rect: Rect, fraction: u16) -> (Rect, Rect) {
    let left_cells: u16 = outer_rect.width / fraction;
    horizontal_left_divide_by_cells(outer_rect, left_cells)
}

/// Divide the screen in two rects
/// Pass it the number of cells the TOP rect should be
pub fn vertical_top_divide_by_cells(outer_rect: Rect, cells: u16) -> (Rect, Rect) {
    let top_rect = Rect {
        x: outer_rect.x,
        y: outer_rect.y,
        width: outer_rect.width,
        height: cells,
    };

    let bottom_rect = Rect {
        x: outer_rect.x,
        y: outer_rect.y + cells,
        width: outer_rect.width,
        height: outer_rect.height - cells,
    };

    (top_rect, bottom_rect)
}

/// Divide the screen in two rects
/// Pass it by how much to divide the TOP part
/// Eg. 3 means the top part is 1/3 of the outer rect, etc.
pub fn vertical_top_divide_by_fraction(outer_rect: Rect, fraction: u16) -> (Rect, Rect) {
    let top_cells: u16 = outer_rect.height / fraction;
    vertical_top_divide_by_cells(outer_rect, top_cells)
}
