/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

<%namespace name="helpers" file="/helpers.mako.rs" />
<% from data import to_rust_ident, ALL_SIDES, PHYSICAL_SIDES, maybe_moz_logical_alias %>

${helpers.four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse",
                               spec="https://drafts.csswg.org/css-backgrounds/#border-color")}

${helpers.four_sides_shorthand("border-style", "border-%s-style",
                               "specified::BorderStyle::parse",
                               spec="https://drafts.csswg.org/css-backgrounds/#border-style")}

<%helpers:shorthand name="border-width" sub_properties="${
        ' '.join('border-%s-width' % side
                 for side in PHYSICAL_SIDES)}"
    spec="https://drafts.csswg.org/css-backgrounds/#border-width">
    use super::parse_four_sides;
    use parser::Parse;
    use values::specified;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (top, right, bottom, left) = try!(parse_four_sides(input, |i| specified::BorderWidth::parse(context, i)));
        Ok(Longhands {
            % for side in PHYSICAL_SIDES:
                ${to_rust_ident('border-%s-width' % side)}: ${side},
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            % for side in PHYSICAL_SIDES:
                let ${side} = self.border_${side}_width.clone();
            % endfor

            super::serialize_four_sides(dest, &top, &right, &bottom, &left)
        }
    }
</%helpers:shorthand>


pub fn parse_border(context: &ParserContext, input: &mut Parser)
                 -> Result<(specified::CSSColor,
                            specified::BorderStyle,
                            specified::BorderWidth), ()> {
    use values::specified::{CSSColor, BorderStyle, BorderWidth};
    let _unused = context;
    let mut color = None;
    let mut style = None;
    let mut width = None;
    let mut any = false;
    loop {
        if color.is_none() {
            if let Ok(value) = input.try(|i| CSSColor::parse(context, i)) {
                color = Some(value);
                any = true;
                continue
            }
        }
        if style.is_none() {
            if let Ok(value) = input.try(|i| BorderStyle::parse(context, i)) {
                style = Some(value);
                any = true;
                continue
            }
        }
        if width.is_none() {
            if let Ok(value) = input.try(|i| BorderWidth::parse(context, i)) {
                width = Some(value);
                any = true;
                continue
            }
        }
        break
    }
    if any {
        Ok((color.unwrap_or_else(|| CSSColor::currentcolor()),
            style.unwrap_or(BorderStyle::none),
            width.unwrap_or(BorderWidth::Medium)))
    } else {
        Err(())
    }
}

% for side, logical in ALL_SIDES:
    <%
        spec = "https://drafts.csswg.org/css-backgrounds/#border-%s" % side
        if logical:
            spec = "https://drafts.csswg.org/css-logical-props/#propdef-border-%s" % side
    %>
    <%helpers:shorthand
        name="border-${side}"
        sub_properties="${' '.join(
            'border-%s-%s' % (side, prop)
            for prop in ['color', 'style', 'width']
        )}"
        alias="${maybe_moz_logical_alias(product, (side, logical), '-moz-border-%s')}"
        spec="${spec}">

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            border_${to_rust_ident(side)}_color: color,
            border_${to_rust_ident(side)}_style: style,
            border_${to_rust_ident(side)}_width: width
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            super::serialize_directional_border(
                dest,
                self.border_${to_rust_ident(side)}_width,
                self.border_${to_rust_ident(side)}_style,
                self.border_${to_rust_ident(side)}_color
            )
        }
    }

    </%helpers:shorthand>
% endfor

<%helpers:shorthand name="border"
    sub_properties="${' '.join('border-%s-%s' % (side, prop)
        for side in PHYSICAL_SIDES
        for prop in ['color', 'style', 'width'])}
        ${' '.join('border-image-%s' % name
        for name in ['outset', 'repeat', 'slice', 'source', 'width'])}
        ${' '.join('-moz-border-%s-colors' % side
        for side in PHYSICAL_SIDES) if product == 'gecko' else ''}"
    spec="https://drafts.csswg.org/css-backgrounds/#border">

    % if product == "gecko":
        use properties::longhands::{_moz_border_top_colors, _moz_border_right_colors,
                                    _moz_border_bottom_colors, _moz_border_left_colors};
    % endif

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        use properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
        use properties::longhands::{border_image_source, border_image_width};

        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            % for side in PHYSICAL_SIDES:
                border_${side}_color: color.clone(),
                border_${side}_style: style,
                border_${side}_width: width.clone(),
                % if product == "gecko":
                    _moz_border_${side}_colors: _moz_border_${side}_colors::get_initial_specified_value(),
                % endif
            % endfor

            // The ‘border’ shorthand resets ‘border-image’ to its initial value.
            // See https://drafts.csswg.org/css-backgrounds-3/#the-border-shorthands
            % for name in "outset repeat slice source width".split():
                border_image_${name}: border_image_${name}::get_initial_specified_value(),
            % endfor
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let all_equal = {
                % for side in PHYSICAL_SIDES:
                  let border_${side}_width = self.border_${side}_width;
                  let border_${side}_style = self.border_${side}_style;
                  let border_${side}_color = self.border_${side}_color;
                % endfor

                border_top_width == border_right_width &&
                border_right_width == border_bottom_width &&
                border_bottom_width == border_left_width &&

                border_top_style == border_right_style &&
                border_right_style == border_bottom_style &&
                border_bottom_style == border_left_style &&

                border_top_color == border_right_color &&
                border_right_color == border_bottom_color &&
                border_bottom_color == border_left_color
            };

            // If all longhands are all present, then all sides should be the same,
            // so we can just one set of color/style/width
            if all_equal {
                super::serialize_directional_border(
                    dest,
                    self.border_${side}_width,
                    self.border_${side}_style,
                    self.border_${side}_color
                )
            } else {
                Ok(())
            }
        }
    }

</%helpers:shorthand>

<%helpers:shorthand name="border-radius" sub_properties="${' '.join(
    'border-%s-radius' % (corner)
     for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
)}" extra_prefixes="webkit" spec="https://drafts.csswg.org/css-backgrounds/#border-radius">
    use values::specified::basic_shape::{BorderRadius, serialize_radius_values};
    use parser::Parse;

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        let radii = try!(BorderRadius::parse(context, input));
        Ok(Longhands {
            border_top_left_radius: radii.top_left,
            border_top_right_radius: radii.top_right,
            border_bottom_right_radius: radii.bottom_right,
            border_bottom_left_radius: radii.bottom_left,
        })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            serialize_radius_values(dest,
                                    &self.border_top_left_radius.0,
                                    &self.border_top_right_radius.0,
                                    &self.border_bottom_right_radius.0,
                                    &self.border_bottom_left_radius.0)
        }
    }
</%helpers:shorthand>

<%helpers:shorthand name="border-image" sub_properties="border-image-outset
    border-image-repeat border-image-slice border-image-source border-image-width"
    extra_prefixes="moz webkit" spec="https://drafts.csswg.org/css-backgrounds-3/#border-image">
    use properties::longhands::{border_image_outset, border_image_repeat, border_image_slice};
    use properties::longhands::{border_image_source, border_image_width};

    pub fn parse_value(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
        % for name in "outset repeat slice source width".split():
            let mut border_image_${name} = border_image_${name}::get_initial_specified_value();
        % endfor

        try!(input.try(|input| {
            % for name in "outset repeat slice source width".split():
                let mut ${name} = None;
            % endfor
            loop {
                if slice.is_none() {
                    if let Ok(value) = input.try(|input| border_image_slice::parse(context, input)) {
                        slice = Some(value);
                        // Parse border image width and outset, if applicable.
                        let maybe_width_outset: Result<_, ()> = input.try(|input| {
                            try!(input.expect_delim('/'));

                            // Parse border image width, if applicable.
                            let w = input.try(|input|
                                border_image_width::parse(context, input)).ok();

                            // Parse border image outset if applicable.
                            let o = input.try(|input| {
                                try!(input.expect_delim('/'));
                                border_image_outset::parse(context, input)
                            }).ok();
                            if w.is_none() && o.is_none() {
                               Err(())
                            }
                            else {
                               Ok((w, o))
                            }
                        });
                        if let Ok((w, o)) = maybe_width_outset {
                            width = w;
                            outset = o;
                        }

                        continue
                    }
                }
                % for name in "source repeat".split():
                    if ${name}.is_none() {
                        if let Ok(value) = input.try(|input| border_image_${name}::parse(context, input)) {
                            ${name} = Some(value);
                            continue
                        }
                    }
                % endfor
                break
            }
            let mut any = false;
            % for name in "outset repeat slice source width".split():
                any = any || ${name}.is_some();
            % endfor
            if any {
                % for name in "outset repeat slice source width".split():
                    if let Some(b_${name}) = ${name} {
                        border_image_${name} = b_${name};
                    }
                % endfor
                Ok(())
            } else {
                Err(())
            }
        }));

        Ok(Longhands {
            % for name in "outset repeat slice source width".split():
                border_image_${name}: border_image_${name},
            % endfor
         })
    }

    impl<'a> ToCss for LonghandsToSerialize<'a>  {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.border_image_source.to_css(dest)?;
            dest.write_str(" ")?;
            self.border_image_slice.to_css(dest)?;
            dest.write_str(" / ")?;
            self.border_image_width.to_css(dest)?;
            dest.write_str(" / ")?;
            self.border_image_outset.to_css(dest)?;
            dest.write_str(" ")?;
            self.border_image_repeat.to_css(dest)
        }
    }
</%helpers:shorthand>
