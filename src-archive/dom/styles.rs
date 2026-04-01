//! DOM Style Module
//! 
//! Provides CSS style manipulation abstractions.

use std::collections::HashMap;

/// CSS property names (type-safe)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CssProperty {
    // Layout
    Display,
    Position,
    Top,
    Right,
    Bottom,
    Left,
    Float,
    Clear,
    
    // Box Model
    Width,
    Height,
    MinWidth,
    MinHeight,
    MaxWidth,
    MaxHeight,
    Padding,
    PaddingTop,
    PaddingRight,
    PaddingBottom,
    PaddingLeft,
    Margin,
    MarginTop,
    MarginRight,
    MarginBottom,
    MarginLeft,
    Border,
    BorderWidth,
    BorderStyle,
    BorderColor,
    BorderRadius,
    
    // Flexbox
    Flex,
    FlexDirection,
    JustifyContent,
    AlignItems,
    AlignContent,
    FlexWrap,
    FlexGrow,
    FlexShrink,
    FlexBasis,
    
    // Grid
    GridTemplateColumns,
    GridTemplateRows,
    GridGap,
    GridColumn,
    GridRow,
    
    // Typography
    Font,
    FontFamily,
    FontSize,
    FontWeight,
    LineHeight,
    TextAlign,
    TextDecoration,
    TextTransform,
    LetterSpacing,
    WhiteSpace,
    
    // Colors
    Color,
    Background,
    BackgroundColor,
    BackgroundImage,
    Opacity,
    
    // Visual
    Overflow,
    OverflowX,
    OverflowY,
    Visibility,
    ZIndex,
    
    // Transitions
    Transition,
    TransitionProperty,
    TransitionDuration,
    TransitionTimingFunction,
    
    // Transform
    Transform,
    
    // Cursor
    Cursor,
    
    // Custom property
    Custom(&'static str),
}

impl CssProperty {
    pub fn as_str(&self) -> &str {
        match self {
            CssProperty::Display => "display",
            CssProperty::Position => "position",
            CssProperty::Top => "top",
            CssProperty::Right => "right",
            CssProperty::Bottom => "bottom",
            CssProperty::Left => "left",
            CssProperty::Float => "float",
            CssProperty::Clear => "clear",
            CssProperty::Width => "width",
            CssProperty::Height => "height",
            CssProperty::MinWidth => "min-width",
            CssProperty::MinHeight => "min-height",
            CssProperty::MaxWidth => "max-width",
            CssProperty::MaxHeight => "max-height",
            CssProperty::Padding => "padding",
            CssProperty::PaddingTop => "padding-top",
            CssProperty::PaddingRight => "padding-right",
            CssProperty::PaddingBottom => "padding-bottom",
            CssProperty::PaddingLeft => "padding-left",
            CssProperty::Margin => "margin",
            CssProperty::MarginTop => "margin-top",
            CssProperty::MarginRight => "margin-right",
            CssProperty::MarginBottom => "margin-bottom",
            CssProperty::MarginLeft => "margin-left",
            CssProperty::Border => "border",
            CssProperty::BorderWidth => "border-width",
            CssProperty::BorderStyle => "border-style",
            CssProperty::BorderColor => "border-color",
            CssProperty::BorderRadius => "border-radius",
            CssProperty::Flex => "flex",
            CssProperty::FlexDirection => "flex-direction",
            CssProperty::JustifyContent => "justify-content",
            CssProperty::AlignItems => "align-items",
            CssProperty::AlignContent => "align-content",
            CssProperty::FlexWrap => "flex-wrap",
            CssProperty::FlexGrow => "flex-grow",
            CssProperty::FlexShrink => "flex-shrink",
            CssProperty::FlexBasis => "flex-basis",
            CssProperty::GridTemplateColumns => "grid-template-columns",
            CssProperty::GridTemplateRows => "grid-template-rows",
            CssProperty::GridGap => "gap",
            CssProperty::GridColumn => "grid-column",
            CssProperty::GridRow => "grid-row",
            CssProperty::Font => "font",
            CssProperty::FontFamily => "font-family",
            CssProperty::FontSize => "font-size",
            CssProperty::FontWeight => "font-weight",
            CssProperty::LineHeight => "line-height",
            CssProperty::TextAlign => "text-align",
            CssProperty::TextDecoration => "text-decoration",
            CssProperty::TextTransform => "text-transform",
            CssProperty::LetterSpacing => "letter-spacing",
            CssProperty::WhiteSpace => "white-space",
            CssProperty::Color => "color",
            CssProperty::Background => "background",
            CssProperty::BackgroundColor => "background-color",
            CssProperty::BackgroundImage => "background-image",
            CssProperty::Opacity => "opacity",
            CssProperty::Overflow => "overflow",
            CssProperty::OverflowX => "overflow-x",
            CssProperty::OverflowY => "overflow-y",
            CssProperty::Visibility => "visibility",
            CssProperty::ZIndex => "z-index",
            CssProperty::Transition => "transition",
            CssProperty::TransitionProperty => "transition-property",
            CssProperty::TransitionDuration => "transition-duration",
            CssProperty::TransitionTimingFunction => "transition-timing-function",
            CssProperty::Transform => "transform",
            CssProperty::Cursor => "cursor",
            CssProperty::Custom(name) => name,
        }
    }
}

/// CSS value types
#[derive(Debug, Clone)]
pub enum CssValue {
    /// Keyword value (auto, none, block, etc.)
    Keyword(&'static str),
    /// Length with unit
    Length(f32, LengthUnit),
    /// Percentage
    Percentage(f32),
    /// Color
    Color(Color),
    /// URL (for background-image, etc.)
    Url(String),
    /// Custom string value
    Custom(String),
}

/// Length units
#[derive(Debug, Clone, Copy)]
pub enum LengthUnit {
    Px,
    Em,
    Rem,
    Percent,
    Vw,
    Vh,
}

impl LengthUnit {
    pub fn as_str(&self) -> &'static str {
        match self {
            LengthUnit::Px => "px",
            LengthUnit::Em => "em",
            LengthUnit::Rem => "rem",
            LengthUnit::Percent => "%",
            LengthUnit::Vw => "vw",
            LengthUnit::Vh => "vh",
        }
    }
}

/// Color representation
#[derive(Debug, Clone, Copy)]
pub enum Color {
    Named(NamedColor),
    Rgb(u8, u8, u8),
    Rgba(u8, u8, u8, f32),
    Hex(u32),
}

/// Named CSS colors
#[derive(Debug, Clone, Copy)]
pub enum NamedColor {
    Transparent,
    Black,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Cyan,
    Magenta,
    Gray,
    Orange,
    Purple,
    Pink,
    Brown,
}

impl Color {
    pub fn as_css_string(&self) -> String {
        match self {
            Color::Named(named) => match named {
                NamedColor::Transparent => "transparent",
                NamedColor::Black => "black",
                NamedColor::White => "white",
                NamedColor::Red => "red",
                NamedColor::Green => "green",
                NamedColor::Blue => "blue",
                NamedColor::Yellow => "yellow",
                NamedColor::Cyan => "cyan",
                NamedColor::Magenta => "magenta",
                NamedColor::Gray => "gray",
                NamedColor::Orange => "orange",
                NamedColor::Purple => "purple",
                NamedColor::Pink => "pink",
                NamedColor::Brown => "brown",
            }.to_string(),
            Color::Rgb(r, g, b) => format!("rgb({}, {}, {})", r, g, b),
            Color::Rgba(r, g, b, a) => format!("rgba({}, {}, {}, {})", r, g, b, a),
            Color::Hex(hex) => format!("#{:06x}", hex),
        }
    }
}

impl CssValue {
    pub fn as_css_string(&self) -> String {
        match self {
            CssValue::Keyword(k) => k.to_string(),
            CssValue::Length(val, unit) => format!("{}{}", val, unit.as_str()),
            CssValue::Percentage(p) => format!("{}%", p),
            CssValue::Color(c) => c.as_css_string(),
            CssValue::Url(u) => format!("url('{}')", u),
            CssValue::Custom(s) => s.clone(),
        }
    }
    
    /// Create pixel length
    pub fn px(val: f32) -> Self {
        CssValue::Length(val, LengthUnit::Px)
    }
    
    /// Create em length
    pub fn em(val: f32) -> Self {
        CssValue::Length(val, LengthUnit::Em)
    }
    
    /// Create rem length
    pub fn rem(val: f32) -> Self {
        CssValue::Length(val, LengthUnit::Rem)
    }
    
    /// Create percentage
    pub fn percent(val: f32) -> Self {
        CssValue::Percentage(val)
    }
}

/// Style builder for fluent API
pub struct StyleBuilder {
    properties: HashMap<CssProperty, CssValue>,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
    
    pub fn property(mut self, prop: CssProperty, value: CssValue) -> Self {
        self.properties.insert(prop, value);
        self
    }
    
    pub fn display(mut self, value: &'static str) -> Self {
        self.properties.insert(CssProperty::Display, CssValue::Keyword(value));
        self
    }
    
    pub fn position(mut self, value: &'static str) -> Self {
        self.properties.insert(CssProperty::Position, CssValue::Keyword(value));
        self
    }
    
    pub fn width<V: Into<CssValue>>(mut self, value: V) -> Self {
        self.properties.insert(CssProperty::Width, value.into());
        self
    }
    
    pub fn height<V: Into<CssValue>>(mut self, value: V) -> Self {
        self.properties.insert(CssProperty::Height, value.into());
        self
    }
    
    pub fn padding<V: Into<CssValue>>(mut self, value: V) -> Self {
        self.properties.insert(CssProperty::Padding, value.into());
        self
    }
    
    pub fn margin<V: Into<CssValue>>(mut self, value: V) -> Self {
        self.properties.insert(CssProperty::Margin, value.into());
        self
    }
    
    pub fn background_color<C: Into<Color>>(mut self, color: C) -> Self {
        self.properties.insert(
            CssProperty::BackgroundColor,
            CssValue::Color(color.into()),
        );
        self
    }
    
    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.properties.insert(CssProperty::Color, CssValue::Color(color.into()));
        self
    }
    
    pub fn font_size<V: Into<CssValue>>(mut self, value: V) -> Self {
        self.properties.insert(CssProperty::FontSize, value.into());
        self
    }
    
    pub fn flex_direction(mut self, value: &'static str) -> Self {
        self.properties.insert(CssProperty::FlexDirection, CssValue::Keyword(value));
        self
    }
    
    pub fn justify_content(mut self, value: &'static str) -> Self {
        self.properties.insert(CssProperty::JustifyContent, CssValue::Keyword(value));
        self
    }
    
    pub fn align_items(mut self, value: &'static str) -> Self {
        self.properties.insert(CssProperty::AlignItems, CssValue::Keyword(value));
        self
    }
    
    /// Build CSS string
    pub fn build(&self) -> String {
        let mut css = String::new();
        css.push('{');
        
        let mut first = true;
        for (prop, value) in &self.properties {
            if !first {
                css.push(';');
            }
            first = false;
            css.push_str(prop.as_str());
            css.push(':');
            css.push_str(&value.as_css_string());
        }
        
        css.push('}');
        css
    }
    
    /// Get properties as HashMap
    pub fn properties(&self) -> &HashMap<CssProperty, CssValue> {
        &self.properties
    }
}

impl Default for StyleBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Style registry for tracking applied styles
pub struct StyleRegistry {
    styles: HashMap<u32, HashMap<CssProperty, CssValue>>,
}

impl StyleRegistry {
    pub fn new() -> Self {
        Self {
            styles: HashMap::new(),
        }
    }
    
    pub fn set_style(&mut self, element_id: u32, prop: CssProperty, value: CssValue) {
        self.styles
            .entry(element_id)
            .or_insert_with(HashMap::new)
            .insert(prop, value);
    }
    
    pub fn get_style(&self, element_id: u32, prop: CssProperty) -> Option<&CssValue> {
        self.styles.get(&element_id)?.get(&prop)
    }
    
    pub fn remove_style(&mut self, element_id: u32, prop: CssProperty) {
        if let Some(styles) = self.styles.get_mut(&element_id) {
            styles.remove(&prop);
        }
    }
    
    pub fn get_all_styles(&self, element_id: u32) -> Option<&HashMap<CssProperty, CssValue>> {
        self.styles.get(&element_id)
    }
}

impl Default for StyleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Common style presets
pub mod presets {
    use super::*;
    
    /// Flexbox center
    pub fn flex_center() -> StyleBuilder {
        StyleBuilder::new()
            .display("flex")
            .justify_content("center")
            .align_items("center")
    }
    
    /// Flexbox row
    pub fn flex_row() -> StyleBuilder {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("row")
    }
    
    /// Flexbox column
    pub fn flex_column() -> StyleBuilder {
        StyleBuilder::new()
            .display("flex")
            .flex_direction("column")
    }
    
    /// Full viewport
    pub fn full_viewport() -> StyleBuilder {
        StyleBuilder::new()
            .width(CssValue::percent(100.0))
            .height(CssValue::Length(100.0, LengthUnit::Vh))
    }
    
    /// Hidden element
    pub fn hidden() -> StyleBuilder {
        StyleBuilder::new()
            .display("none")
    }
    
    /// Block element
    pub fn block() -> StyleBuilder {
        StyleBuilder::new()
            .display("block")
    }
    
    /// Inline-block element
    pub fn inline_block() -> StyleBuilder {
        StyleBuilder::new()
            .display("inline-block")
    }
    
    /// Absolute positioning
    pub fn absolute() -> StyleBuilder {
        StyleBuilder::new()
            .position("absolute")
    }
    
    /// Fixed positioning
    pub fn fixed() -> StyleBuilder {
        StyleBuilder::new()
            .position("fixed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_css_property_names() {
        assert_eq!(CssProperty::Display.as_str(), "display");
        assert_eq!(CssProperty::BackgroundColor.as_str(), "background-color");
        assert_eq!(CssProperty::FlexDirection.as_str(), "flex-direction");
    }
    
    #[test]
    fn test_css_value_conversion() {
        let px = CssValue::px(16.0);
        assert_eq!(px.as_css_string(), "16px");
        
        let em = CssValue::em(1.5);
        assert_eq!(em.as_css_string(), "1.5em");
        
        let percent = CssValue::percent(50.0);
        assert_eq!(percent.as_css_string(), "50%");
    }
    
    #[test]
    fn test_color_conversion() {
        let rgb = Color::Rgb(255, 0, 0);
        assert_eq!(rgb.as_css_string(), "rgb(255, 0, 0)");
        
        let hex = Color::Hex(0xFF0000);
        assert_eq!(hex.as_css_string(), "#ff0000");
    }
    
    #[test]
    fn test_style_builder() {
        let style = StyleBuilder::new()
            .display("flex")
            .justify_content("center")
            .align_items("center")
            .build();
        
        assert!(style.contains("display:flex"));
        assert!(style.contains("justify-content:center"));
        assert!(style.contains("align-items:center"));
    }
}
