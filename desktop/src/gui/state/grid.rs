
use iced::{Element, Length as IcedLength, Rectangle, Renderer as DefaultRenderer, Size, Theme as DefaultTheme};
use iced::alignment::{Horizontal, Vertical};
use iced::advanced::{Renderer as RendererT, Widget};
use iced::advanced::layout::{Layout, Limits, Node};
use iced::advanced::layout::flex::{self, Axis};
use iced::advanced::mouse::{Cursor, Interaction};
use iced::advanced::overlay;
use iced::advanced::renderer::{Style};
use iced::advanced::widget::{Tree};

use crate::unwrap_variant;

#[derive(Clone, Copy)]
enum Line {
    Row,
    Column
}

enum Info {
    ConcreteLength(f32),
    FillPortion(u16)
}

#[derive(Clone, Copy)]
enum Length {
    Fixed(f32),
    Shrink { length: Option<f32> },
    Fill { portions: u16, length: Option<f32> }
}

impl Length {
    fn from_iced_length(iced_length: IcedLength) -> Self {
        match iced_length {
            IcedLength::Fixed(length) => Self::Fixed(length),
            IcedLength::Shrink => Self::Shrink { length: None },
            IcedLength::Fill => Self::Fill { portions: 1, length: None },
            IcedLength::FillPortion(portions) => Self::Fill { portions, length: None }
        }
    }

    fn into_iced_length(&self) -> IcedLength {
        match self {
            Self::Fixed(length) => IcedLength::Fixed(*length),
            Self::Shrink { .. } => IcedLength::Shrink,
            Self::Fill { portions, .. } => IcedLength::FillPortion(*portions)
        }
    }

    // If Length is Shrink or Fill, `length` must be Some
    fn get_length(&self) -> f32 {
        match self {
            Self::Fixed(length) => *length,
            Self::Shrink { length: Some(len) } => *len,
            Self::Fill { length: Some(len), ..} => *len,
            _ => panic!("`length` is None; must be Some when calling this function")
        }
    }

    fn fill_portions(&self) -> u16 {
        match self {
            Self::Fixed(_) | Self::Shrink { .. } => 0,
            Self::Fill { portions, .. } => *portions
        }
    }
}

#[derive(Clone, Copy)]
struct RowConfig {
    alignment: Vertical,
    height: Length
}

impl Default for RowConfig {
    fn default() -> Self {
        Self {
            alignment: Vertical::Top,
            height: Length::Shrink { length: None }
        }
    }
}

#[derive(Clone, Copy)]
struct ColumnConfig {
    alignment: Horizontal,
    width: Length
}

impl Default for ColumnConfig {
    fn default() -> Self {
        Self {
            alignment: Horizontal::Left,
            width: Length::Shrink { length: None }
        }
    }
}


pub struct Grid<'a, Message, Theme = DefaultTheme, Renderer = DefaultRenderer> {
    elements: Vec<Element<'a, Message, Theme, Renderer>>,
    row_configs: Vec<RowConfig>,
    column_configs: Vec<ColumnConfig>,
    width: IcedLength,
    height: IcedLength,
    spacing_x: f32,
    spacing_y: f32,
    padding: f32
}

impl<'a, Message, Theme, Renderer> Grid<'a, Message, Theme, Renderer> 
where 
    Message: 'a,
    Renderer: RendererT + 'a,
    Theme: 'a
{
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            row_configs: Vec::new(),
            column_configs: Vec::new(),
            width: IcedLength::Shrink,
            height: IcedLength::Shrink,
            spacing_x: 0f32,
            spacing_y: 0f32,
            padding: 0f32
        }
    }
    pub fn with<E>(elements: impl IntoIterator<Item = E>, column_amount: usize) -> Self 
    where 
        E: Into<Element<'a, Message, Theme, Renderer>>,
    {
        let elements_iter = elements.into_iter();
        let row_amount = elements_iter.size_hint().0 / column_amount;
        let mut row_configs: Vec<RowConfig> = Vec::with_capacity(row_amount);
        let mut column_configs = vec![ColumnConfig::default(); column_amount];
        let mut elements = Vec::with_capacity(row_amount * column_amount);
        let mut grid_width = IcedLength::Shrink;
        let mut grid_height = IcedLength::Shrink;

        for (child_i, child) in elements_iter.enumerate() {
            let child: Element<'_, _, Theme, Renderer> = child.into();
            let child_size = child.as_widget().size();

            let (row_i, col_i) = (child_i / column_amount, child_i % column_amount);  

            let row_height = match row_configs.get_mut(row_i) {
                Some(row_config) => &mut row_config.height,
                None => {
                    row_configs.push(RowConfig::default());

                    &mut row_configs[row_i].height
                } 
            };

            into_highest_fill_portion(row_height, &child_size.height);

            let column_width = &mut column_configs
                .get_mut(col_i)
                .expect("Row length must be equal to the amount of columns")
                .width;         

            into_highest_fill_portion(column_width, &child_size.width);

            elements.push(child);      
        }

        assert!(elements.len() == row_amount * column_amount, 
        "The amount of elements must be equal to the product of amount of rows and columns");

        Self {
            elements,
            row_configs,
            column_configs,
            width: grid_width,
            height: grid_height,
            spacing_x: 0f32,
            spacing_y: 0f32,
            padding: 0f32
        }
    } 

    pub fn push_row(
        &mut self, 
        row: impl IntoIterator<Item = Element<'a, Message, Theme, Renderer>>
    ) -> &mut Self 
    {
        self.row_configs.push(RowConfig::default());

        let pre_elements_amount = self.elements.len();

        for (col_i, child) in row.into_iter().enumerate() {
            let child_size = child.as_widget().size();

            let row_height = &mut self.row_configs
                .last_mut()
                .expect("Vec should never be empty due to pushing in the start of the function")
                .height;

            into_highest_fill_portion(row_height, &child_size.height);

            let column_width = &mut self.column_configs
                .get_mut(col_i)
                .expect("The length of the given row must be the same as the previous rows")
                .width;
            
            into_highest_fill_portion(column_width, &child_size.width);

            self.elements.push(child);
        }

        let added_elements_amount = self.elements.len() - pre_elements_amount;

        if added_elements_amount != self.column_configs.len() {
            panic!("The length of the given row must be the same as the previous rows");
        }

        self
    } 

    pub fn align_row(&mut self, row_i: usize, alignment: Vertical) -> &mut Self {
        self.row_configs[row_i].alignment = alignment;

        self
    }

    pub fn align_column(&mut self, column_i: usize, alignment: Horizontal) -> &mut Self {
        self.column_configs[column_i].alignment = alignment;

        self
    }

    pub fn set_spacing_x(&mut self, spacing_x: f32) -> &mut Self {
        self.spacing_x = spacing_x;
        
        self
    }

    pub fn set_spacing_y(&mut self, spacing_y: f32) -> &mut Self {
        self.spacing_y = spacing_y;
        
        self
    }

    /* 
    fn remaining_space_and_total_fill_portions(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> (Size, Size<usize>)
    where
        Renderer: RendererT,
    {
        let columns_amount = self.column_configs.len();

        let mut accumulate_line = 
            |line, total_length: &mut f32, total_fill_portions: &mut u16, index, length: &Length| 
        {
            let info = info_of_line(
                &mut self.elements, length, line, index, tree, renderer, columns_amount
            );
            
            match info {
                Info::ConcreteLength(length) => *total_length += length,
                Info::FillPortion(portions) => *total_fill_portions += portions
            }
        };

        let (mut total_height, mut total_height_fill_portions) = (0f32, 0u16);

        for (row_i, row_height) in self.row_configs
            .iter()
            .map(|row_config| &row_config.height)
            .enumerate() 
        {
            accumulate_line(
                Line::Row, &mut total_height, &mut total_height_fill_portions, row_i, row_height
            );
        }

        let (mut total_width, mut total_width_fill_portions) = (0f32, 016);

        for (col_i, column_width) in self.column_configs
            .iter()
            .map(|col_config| &col_config.width)
            .enumerate()
        {
            accumulate_line(
                Line::Column, &mut total_width, &mut total_width_fill_portions, col_i, column_width
            );
        }

        let total_size = Size::new(total_width, total_height);
        let remaining_space = limits.max() - total_size;
        let total_fill_portions = Size::new(
            total_width_fill_portions as usize, total_height_fill_portions as usize
        );

        (remaining_space, total_fill_portions)
    }
    */

    fn concretize_lengths_of_all_lines(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) {
        let columns_amount = self.column_configs.len();

        let (total_resolved_height, total_height_fill_portions, total_height_fill_amount) = {
            let row_heights = get_row_heights(&mut self.row_configs);
            
            handle_non_fill(
                &mut self.elements, 
                self.spacing_y, row_heights, Line::Row, tree, renderer, columns_amount
            )
        };

        let (total_resolved_width, total_width_fill_portions, total_width_fill_amount) = {
            let column_widths = get_column_widths(&mut self.column_configs);
        
            handle_non_fill(
                &mut self.elements, 
                self.spacing_x, column_widths, Line::Column, tree, renderer, columns_amount
            )
        };

        if total_height_fill_amount != 0 {
            let remaining_height = limits.max().height 
                - total_resolved_height 
                - self.spacing_y * (total_height_fill_amount - 1) as f32;

            let row_heights = get_row_heights(&mut self.row_configs);
            handle_fill(row_heights, remaining_height, total_height_fill_portions);  
        }

        if total_width_fill_amount != 0 {
            let remaining_width = limits.max().width 
                - total_resolved_width
                - self.spacing_x * (total_height_fill_portions - 1) as f32;

            let column_widths = get_column_widths(&mut self.column_configs);
            handle_fill(column_widths, remaining_width, total_width_fill_portions);
        }

    }
}


impl<'a, Message, Theme, Renderer> From<Grid<'a, Message, Theme, Renderer>> 
    for Element<'a, Message, Theme, Renderer> 
where 
    Message: 'a,
    Renderer: RendererT + 'a,
    Theme: 'a
{
    fn from(value: Grid<'a, Message, Theme, Renderer>) -> Self {
        Self::new(value)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Grid<'a, Message, Theme, Renderer> 
where 
    Message: 'a,
    Renderer: RendererT + 'a,
    Theme: 'a
{
    fn children(&self) -> Vec<Tree> {
        self.elements
            .iter()
            .map(Tree::new)
            .collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.elements);
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        for ((child, tree), layout) in self
            .elements
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child.as_widget().draw(
                tree, renderer, theme, style, layout, cursor, viewport,
            );
        }        
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node
    {   
        //let (mut remaining_fill_space, total_fill_portions) = 
          //  self.remaining_space_and_total_fill_portions(tree, renderer, limits);

        self.concretize_lengths_of_all_lines(tree, renderer, limits);


        let rows_amount = self.row_configs.len();
        let columns_amount = self.column_configs.len();
        
        let mut nodes = Vec::with_capacity(self.elements.len());
        let mut grid_size = Size::ZERO;
        let mut remaining_height = limits.max().height;
        let mut y_pos = 0f32; 

        for row_i in 0..rows_amount {
            if remaining_height == 0.0 {
                break;
            }

            let row_config = &self.row_configs[row_i];

            let cell_height = {
                let row_concrete_height = row_config.height.get_length();

                row_concrete_height.min(remaining_height)
            };

            if cell_height == 0.0 {
                continue;
            }

            remaining_height -= cell_height;

            let mut remaining_width = limits.max().width;
            let mut x_pos = 0f32;

            for col_i in 0..columns_amount {
                if remaining_width == 0.0 {
                    break;
                }

                let column_config = &self.column_configs[col_i];

                let cell_width = {
                    let column_concrete_width = column_config.width.get_length();

                    column_concrete_width.min(remaining_width)
                };

                if cell_width == 0.0 {
                    continue;
                }

                remaining_width -= cell_width;

                let node = {
                    let max_size = Size::new(cell_width, cell_height);
                    let limits = Limits::new(Size::ZERO, max_size);
                    let cell_i = row_i * columns_amount + col_i;
                    let child_tree = &mut tree.children[cell_i];

                    self.elements[cell_i]
                        .as_widget_mut()
                        .layout(child_tree, renderer, &limits)
                        .move_to((x_pos, y_pos))
                        .align(column_config.alignment.into(), row_config.alignment.into(), max_size)
                };  

                nodes.push(node);

                x_pos += cell_width + self.spacing_x;       
            }
            grid_size.width = grid_size.width.max(x_pos - self.spacing_x);

            y_pos += cell_height + self.spacing_y;
        }

        grid_size.height = y_pos - self.spacing_y;

        Node::with_children(grid_size, nodes)
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> Interaction
    {
        self.elements
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(tree, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    )
    {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.elements
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>>
    {
        overlay::from_children(
            &mut self.elements,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    fn size(&self) -> Size<IcedLength> {
        Size {
            width: self.width,
            height: self.height
        }
    }    

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn iced::advanced::Clipboard,
        shell: &mut iced::advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    )
    {
        for ((child, tree), layout) in self
            .elements
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                tree, event, layout, cursor, renderer, clipboard, shell,
                viewport,
            );
        }        
    }
}

struct Cell<'a, 'b, Message, Theme = DefaultTheme, Renderer = DefaultRenderer> {
    element: &'a mut Element<'b, Message, Theme, Renderer>,
    tree: &'a mut Tree
}

impl<'a, 'b, Message, Theme, Renderer> Cell<'a, 'b, Message, Theme, Renderer> 
where
    Renderer: RendererT
{
    fn create(element: &'a mut Element<'b, Message, Theme, Renderer>, tree: &'a mut Tree) -> Self {
        debug_assert_eq!(
            element.as_widget().tag(),
            tree.tag,
            "Cell created with mismatched element/tree tag"
        );

        Self {
            element,
            tree
        }
    }


    fn create_iter(
        elements: impl IntoIterator<Item = &'a mut Element<'b, Message, Theme, Renderer>>,
        trees: impl IntoIterator<Item = &'a mut Tree>
    ) -> impl Iterator<Item = Self>
    {
        elements
            .into_iter()
            .zip(trees.into_iter())
            .map(|(element, tree)| Self::create(element, tree))
    }
    fn concrete_size(self, limits: &Limits, renderer: &Renderer) -> Size {
        self.element
            .as_widget_mut()
            .layout(self.tree, renderer, limits)
            .size()
    }
} 

fn max_span_in_line<'a, 'b, Message, Renderer, Theme>(
    cells_in_line: impl Iterator<Item = (&'a mut Element<'b, Message, Theme, Renderer>, &'a mut Tree)>,
    line: Line,
    renderer: &Renderer
) -> f32 
where 
    'b: 'a,
    Message: 'b,
    Renderer: RendererT + 'b,
    Theme: 'b
{
    let limits = Limits::with_compression(
        Size::ZERO, 
        Size::INFINITE, 
        Size::new(true, true)
    );

    cells_in_line
        .map(|(cell, tree)| {
            let cell_size = cell.as_widget_mut()
                .layout(tree, renderer, &limits)
                .size();

            match line {
                Line::Row => cell_size.height,
                Line::Column => cell_size.width

            }
        })
        .reduce(f32::max)
        .expect("Line should never be empty")
}

fn highest_concrete_length<'a, 'b, Message, Theme, Renderer>(
    cells: impl IntoIterator<Item = Cell<'a, 'b, Message, Theme, Renderer>>,
    mut relevant_dimension: impl FnMut(Size) -> f32,
    limits: &Limits,
    renderer: &Renderer
) -> f32
where
    'b: 'a,
    Message: 'a,
    Renderer: RendererT + 'a,
    Theme: 'a
{
    cells.into_iter()
        .map(|cell| {
            let size = cell.concrete_size(limits, renderer);

            relevant_dimension(size)
        })
        .reduce(f32::max)
        .expect("Cells should never be empty")
}

// Returns Some if `length` is not Fill and None otherwise
fn non_fill_length(length: Length) -> Option<Length> {
    match length {
        Length::Fill { .. } => None,
        _ => Some(length)
    }
}


fn into_highest_fill_portion(old: &mut Length, new: &IcedLength) {
    let new_fill_portion = new.fill_factor();

    if new_fill_portion == 0 {
        return;
    } else {
        let old_fill_portion = old.fill_portions();
        let fill_portions = old_fill_portion.max(new_fill_portion);

        *old = Length::Fill { portions: fill_portions, length: None }
    }
}



fn handle_non_fill<'a, Message, Theme, Renderer>(
    elements: &mut Vec<Element<'_, Message, Theme, Renderer>>,
    spacing: f32,
    lengths: impl Iterator<Item = &'a mut Length>,
    line: Line,
    tree: &mut Tree,
    renderer: &Renderer,
    columns_amount: usize
) -> (f32, u16, u16) 
where 
    Renderer: RendererT
{   
    let mut total_resolved_length = 0f32;
    let mut total_fill_portions = 0;
    let mut total_fill_amount = 0;

    for (i, length) in lengths.enumerate() {
        match length {
            Length::Fixed(fixed_length) => total_resolved_length += *fixed_length + spacing,
            Length::Shrink { length } => {
                let limits = Limits::with_compression(
                    Size::ZERO, Size::INFINITE, Size::new(false, false)
                );

                let shrink_length = match line {
                    Line::Row => {
                        let cells_in_row = get_cells_in_row(elements, tree, i, columns_amount);

                        let height_of = |size: Size| size.height;

                        highest_concrete_length(cells_in_row, height_of, &limits, renderer)
                    },
                    Line::Column => {
                        let cells_in_col = {
                            let elements_in_col = elements
                                .iter_mut()
                                .skip(i)
                                .step_by(columns_amount);

                            let trees_in_col = tree.children
                                .iter_mut()
                                .skip(i)
                                .step_by(columns_amount);

                            Cell::create_iter(elements_in_col, trees_in_col)
                        };

                        let width_of = |size: Size| size.width;

                        highest_concrete_length(cells_in_col, width_of, &limits, renderer)
                    }
                };

                *length = Some(shrink_length);
                total_resolved_length += shrink_length + spacing;
            },
            Length::Fill { portions, .. } => {
                total_fill_portions += *portions;
                total_fill_amount += 1;
            },
        };
    }

    (total_resolved_length, total_fill_portions, total_fill_amount)
}

fn handle_fill<'a>(
    lengths: impl Iterator<Item = &'a mut Length>,
    remaining_length: f32,
    total_fill_portions: u16,
) {
    for length in lengths {
        if let Length::Fill { portions, length } = length {
            let concrete_length = remaining_length * *portions as f32 / total_fill_portions as f32;

            *length = Some(concrete_length);
        }
    }
}

fn get_cells_in_row<'a, 'b, Message, Theme, Renderer>(
    elements: &'a mut Vec<Element<'b, Message, Theme, Renderer>>,
    tree: &'a mut Tree,
    row_i: usize,
    columns_amount: usize
) -> impl Iterator<Item = Cell<'a, 'b, Message, Theme, Renderer>>
where
    'b: 'a,
    Renderer: RendererT 
{
    let range = {
        let offset = row_i * columns_amount;
        offset..offset + columns_amount
    };

    Cell::create_iter(
        &mut elements[range.clone()], &mut tree.children[range]
    )
}

fn info_of_line<Message, Theme, Renderer>(
    elements: &mut Vec<Element<'_, Message, Theme, Renderer>>,
    length: &mut Length,
    line: Line,
    line_i: usize,
    tree: &mut Tree,
    renderer: &Renderer,
    columns_amount: usize
) -> Info 
where 
    Renderer: RendererT
{   
    match length {
        Length::Fixed(fixed_length) => Info::ConcreteLength(*fixed_length),
        Length::Shrink { length } => {
            let limits = Limits::with_compression(
                Size::ZERO, Size::INFINITE, Size::new(false, false)
            );

            let shrink_length = match line {
                Line::Row => {
                    let range = {
                        let offset = line_i * columns_amount;
                        offset..offset + columns_amount
                    };

                    let cells_in_row = Cell::create_iter(
                        &mut elements[range.clone()], &mut tree.children[range]
                    );

                    let height_of = |size: Size| size.height;

                    highest_concrete_length(cells_in_row, height_of, &limits, renderer)
                },
                Line::Column => {
                    let cells_in_col = {
                        let elements_in_col = elements
                            .iter_mut()
                            .skip(line_i)
                            .step_by(columns_amount);

                        let trees_in_col = tree.children
                            .iter_mut()
                            .skip(line_i)
                            .step_by(columns_amount);

                        Cell::create_iter(elements_in_col, trees_in_col)
                    };

                    let width_of = |size: Size| size.width;

                    highest_concrete_length(cells_in_col, width_of, &limits, renderer)
                }
            };

            *length = Some(shrink_length);

            Info::ConcreteLength(shrink_length)
        },
        Length::Fill { portions, .. } => Info::FillPortion(*portions),
    }        
}
/* 

fn concrete_length<Message, Theme, Renderer>(
    length: &Length,
    line: Line,
    line_i: usize,
    tree: &mut Tree,
    elements: &mut Vec<Element<'_, Message, Theme, Renderer>>,
    remaining_space: &mut f32,
    remaining_fill_space: &mut f32,
    total_fill_portions: usize,
    renderer: &Renderer,
    columns_amount: usize,
) -> f32 
where
    Renderer: RendererT
{
    let info = info_of_line(
        elements, length, line, line_i, tree, renderer, columns_amount
    );

    match info {
        Info::ConcreteLength(mut length) => {
            length = remaining_space.min(length);
            *remaining_space -= length;
            length
        }
        Info::FillPortion(portions) => {
            if *remaining_fill_space <= 0.0 {
                return 0.0;
            }
            let length = *remaining_fill_space * portions as f32 / total_fill_portions as f32;
            *remaining_space -= length;
            *remaining_fill_space -= length;
            length
        }
    }
}
*/

fn get_row_heights(
    row_configs: &mut Vec<RowConfig>
) -> impl Iterator<Item = &mut Length>
{
    row_configs
        .iter_mut()
        .map(|row_config| &mut row_config.height)
}

fn get_column_widths(
    column_configs: &mut Vec<ColumnConfig>
) -> impl Iterator<Item = &mut Length>
{
    column_configs
        .iter_mut()
        .map(|column_config| &mut column_config.width)
}