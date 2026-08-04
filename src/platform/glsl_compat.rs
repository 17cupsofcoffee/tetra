//! Translates Tetra's OpenGL-flavoured GLSL into the Vulkan-flavoured GLSL
//! that Naga's frontend understands.
//!
//! Naga only accepts GLSL 440+, and requires explicit `layout(location = X)`
//! and `layout(binding = X)` qualifiers on all inputs, outputs and uniforms.
//! It also has no support for combined `sampler2D` uniforms - textures and
//! samplers have to be declared seperately, and combined at the call site
//! via a constructor (e.g. `texture(sampler2D(t, s), uv)`).
//!
//! Rather than pushing all of that onto the user, this module rewrites the
//! old-style shader source:
//!
//! * The `#version` directive is replaced with `#version 450`.
//! * Vertex attributes are given fixed locations, based on their names.
//! * Varyings are given locations shared between the two stages, based on
//!   their names.
//! * Uniforms are given bindings shared between the two stages, based on
//!   their names.
//! * `sampler2D`-style uniforms are split into a texture and a sampler,
//!   with a `#define` that reconstructs the combined object at usage sites.
//! * `gl_InstanceID` and `gl_FragColor` are mapped to their modern
//!   equivalents.

use hashbrown::HashMap;

pub const TEXTURE_SUFFIX: &str = "_tetra_tex";
pub const SAMPLER_SUFFIX: &str = "_tetra_smp";

/// The fixed attribute locations that Tetra's vertex data is bound to.
pub const ATTRIBUTES: [(&str, u32); 3] = [("a_position", 0), ("a_uv", 1), ("a_color", 2)];

#[derive(Debug, Clone, Default)]
pub struct ShaderInterface {
    /// Data uniforms, and the bindings they were assigned.
    pub uniforms: Vec<(String, u32)>,

    /// Combined texture/samplers, and the (texture, sampler) bindings they
    /// were assigned.
    pub samplers: Vec<(String, u32, u32)>,
}

pub struct ProcessedShaders {
    pub vertex: String,
    pub fragment: String,
    pub interface: ShaderInterface,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage {
    Vertex,
    Fragment,
}

#[derive(Default)]
struct SharedTables {
    uniform_bindings: HashMap<String, u32>,
    sampler_bindings: HashMap<String, (u32, u32)>,
    next_binding: u32,

    varying_locations: HashMap<String, u32>,
    next_varying: u32,

    next_attribute: u32,
    next_frag_output: u32,

    interface: ShaderInterface,
}

impl SharedTables {
    fn new() -> SharedTables {
        SharedTables {
            next_attribute: ATTRIBUTES.len() as u32,
            interface: ShaderInterface {
                uniforms: Vec::new(),
                samplers: Vec::new(),
            },
            ..Default::default()
        }
    }

    fn uniform_binding(&mut self, name: &str) -> u32 {
        if let Some(&binding) = self.uniform_bindings.get(name) {
            return binding;
        }

        let binding = self.next_binding;
        self.next_binding += 1;
        self.uniform_bindings.insert(name.to_owned(), binding);
        self.interface.uniforms.push((name.to_owned(), binding));

        binding
    }

    fn sampler_binding(&mut self, name: &str) -> (u32, u32) {
        if let Some(&bindings) = self.sampler_bindings.get(name) {
            return bindings;
        }

        let bindings = (self.next_binding, self.next_binding + 1);
        self.next_binding += 2;
        self.sampler_bindings.insert(name.to_owned(), bindings);
        self.interface
            .samplers
            .push((name.to_owned(), bindings.0, bindings.1));

        bindings
    }

    fn varying_location(&mut self, name: &str) -> u32 {
        if let Some(&location) = self.varying_locations.get(name) {
            return location;
        }

        let location = self.next_varying;
        self.next_varying += 1;
        self.varying_locations.insert(name.to_owned(), location);

        location
    }

    fn frag_output_location(&mut self) -> u32 {
        let location = self.next_frag_output;
        self.next_frag_output += 1;

        location
    }

    fn attribute_location(&mut self, name: &str) -> u32 {
        for (attr_name, location) in ATTRIBUTES {
            if attr_name == name {
                return location;
            }
        }

        let location = self.next_attribute;
        self.next_attribute += 1;

        location
    }
}

pub fn process(vertex: &str, fragment: &str) -> ProcessedShaders {
    let mut tables = SharedTables::new();

    let vertex = process_stage(vertex, Stage::Vertex, &mut tables);
    let fragment = process_stage(fragment, Stage::Fragment, &mut tables);

    ProcessedShaders {
        vertex,
        fragment,
        interface: tables.interface,
    }
}

fn process_stage(source: &str, stage: Stage, tables: &mut SharedTables) -> String {
    let source = strip_comments(source);

    let mut output = String::with_capacity(source.len() + 256);

    output.push_str("#version 450\n");
    output.push_str("#define gl_InstanceID int(gl_InstanceIndex)\n");

    if stage == Stage::Fragment && contains_word(&source, "gl_FragColor") {
        output.push_str("layout(location = 0) out vec4 tetra_frag_color;\n");
        output.push_str("#define gl_FragColor tetra_frag_color\n");
    }

    for item in split_items(&source) {
        match item {
            Item::Directive(line) => {
                let trimmed = line.trim_start();
                if !trimmed.starts_with("#version") {
                    output.push_str(line);
                }
                output.push('\n');
            }

            Item::Statement(text) => {
                process_statement(text, stage, tables, &mut output);
            }

            Item::Block(text) => {
                output.push_str(text);
                output.push('\n');
            }
        }
    }

    output
}

enum Item<'a> {
    /// A preprocessor directive (one line).
    Directive(&'a str),

    /// A top-level statement, terminated by a semicolon, not containing
    /// any braces (e.g. a declaration).
    Statement(&'a str),

    /// Anything containing braces (function definitions, interface blocks),
    /// passed through untouched.
    Block(&'a str),
}

fn split_items(source: &str) -> Vec<Item<'_>> {
    let mut items = Vec::new();
    let bytes = source.as_bytes();

    let mut i = 0;
    let mut start = 0;
    let mut depth = 0usize;
    let mut has_braces = false;
    let mut at_line_start = true;

    while i < bytes.len() {
        let b = bytes[i];

        if depth == 0 && at_line_start && source[start..i].trim().is_empty() && b == b'#' {
            // A preprocessor directive - consume the rest of the line.
            let line_end = source[i..]
                .find('\n')
                .map(|offset| i + offset)
                .unwrap_or(bytes.len());

            if !source[start..i].trim().is_empty() {
                items.push(Item::Block(&source[start..i]));
            }

            items.push(Item::Directive(&source[i..line_end]));

            i = line_end + 1;
            start = i;
            at_line_start = true;
            has_braces = false;
            continue;
        }

        match b {
            b'{' => {
                depth += 1;
                has_braces = true;
            }
            b'}' => {
                depth = depth.saturating_sub(1);

                if depth == 0 {
                    // End of a function definition or similar - check whether
                    // it's followed by a declarator/semicolon (e.g. an
                    // interface block), and if so, keep going until the
                    // semicolon.
                    let rest = source[i + 1..].trim_start();
                    if !(rest.starts_with(';') || rest.chars().next().is_some_and(is_ident_char)) {
                        items.push(Item::Block(&source[start..=i]));
                        start = i + 1;
                    }
                }
            }
            b';' if depth == 0 => {
                let text = &source[start..=i];
                if has_braces {
                    items.push(Item::Block(text));
                } else if !text.trim().is_empty() {
                    items.push(Item::Statement(text));
                }
                start = i + 1;
                has_braces = false;
            }
            _ => {}
        }

        at_line_start = b == b'\n';
        i += 1;
    }

    if !source[start..].trim().is_empty() {
        items.push(Item::Block(&source[start..]));
    }

    items
}

fn process_statement(text: &str, stage: Stage, tables: &mut SharedTables, output: &mut String) {
    let words: Vec<&str> = tokenize(text);

    // Statements that already have layout qualifiers are assumed to be
    // written in the new style, and are passed through untouched.
    if words.first() == Some(&"layout") {
        output.push_str(text);
        output.push('\n');
        return;
    }

    const INTERPOLATION_QUALIFIERS: [&str; 6] = [
        "flat",
        "noperspective",
        "smooth",
        "centroid",
        "invariant",
        "precise",
    ];

    let mut qualifier_count = 0;
    while words
        .get(qualifier_count)
        .is_some_and(|w| INTERPOLATION_QUALIFIERS.contains(w))
    {
        qualifier_count += 1;
    }

    match words.get(qualifier_count) {
        Some(&"uniform") => {
            let Some(&ty) = words.get(qualifier_count + 1) else {
                output.push_str(text);
                output.push('\n');
                return;
            };

            if let Some(dim) = ty.strip_prefix("sampler") {
                if !dim.is_empty() && !ty.contains("Shadow") {
                    // A combined texture/sampler - split it in two.
                    for name in declarator_names(&words[qualifier_count + 2..]) {
                        let (tex_binding, smp_binding) = tables.sampler_binding(name);

                        output.push_str(&format!(
                            "layout(set = 0, binding = {tex_binding}) uniform texture{dim} {name}{TEXTURE_SUFFIX};\n",
                        ));
                        output.push_str(&format!(
                            "layout(set = 0, binding = {smp_binding}) uniform sampler {name}{SAMPLER_SUFFIX};\n",
                        ));
                        output.push_str(&format!(
                            "#define {name} {ty}({name}{TEXTURE_SUFFIX}, {name}{SAMPLER_SUFFIX})\n",
                        ));
                    }

                    return;
                }
            }

            // A data uniform - all declarators share the statement, but each
            // one needs its own binding, so the statement gets split up.
            let names: Vec<&str> = declarator_names(&words[qualifier_count + 2..]).collect();

            if names.is_empty() {
                output.push_str(text);
                output.push('\n');
                return;
            }

            for declarator in split_declarators(text, ty) {
                let name = names
                    .iter()
                    .find(|n| contains_word(&declarator, n))
                    .copied()
                    .unwrap_or(names[0]);

                let binding = tables.uniform_binding(name);

                output.push_str(&format!(
                    "layout(set = 0, binding = {binding}) uniform {ty} {declarator};\n"
                ));
            }
        }

        Some(&direction @ ("in" | "out" | "attribute" | "varying")) => {
            let Some(&name) = words.last() else {
                output.push_str(text);
                output.push('\n');
                return;
            };

            let name = name.trim_end_matches(|c: char| !is_ident_char(c));

            let location = match (stage, direction) {
                (Stage::Vertex, "in" | "attribute") => tables.attribute_location(name),
                (Stage::Vertex, _) => tables.varying_location(name),
                (Stage::Fragment, "in" | "varying") => tables.varying_location(name),
                (Stage::Fragment, "out") => tables.frag_output_location(),
                _ => unreachable!(),
            };

            // `attribute`/`varying` aren't valid in GLSL 450, so they're
            // rewritten to `in`/`out`.
            let modernized = match direction {
                "attribute" => text.replacen("attribute", "in", 1),
                "varying" => match stage {
                    Stage::Vertex => text.replacen("varying", "out", 1),
                    Stage::Fragment => text.replacen("varying", "in", 1),
                },
                _ => text.to_owned(),
            };

            output.push_str(&format!("layout(location = {location}) {modernized}\n"));
        }

        _ => {
            output.push_str(text);
            output.push('\n');
        }
    }
}

fn tokenize(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace() || c == ';' || c == ',')
        .filter(|w| !w.is_empty())
        .collect()
}

fn declarator_names<'a>(words: &'a [&'a str]) -> impl Iterator<Item = &'a str> + 'a {
    words.iter().map(|w| {
        let end = w.find(|c: char| !is_ident_char(c)).unwrap_or(w.len());
        &w[..end]
    })
}

fn split_declarators(text: &str, ty: &str) -> Vec<String> {
    // Everything after the type word, up to the semicolon, split on commas.
    let after_ty = match text.find(ty) {
        Some(pos) => &text[pos + ty.len()..],
        None => text,
    };

    after_ty
        .trim_end()
        .trim_end_matches(';')
        .split(',')
        .map(|d| d.trim().to_owned())
        .filter(|d| !d.is_empty())
        .collect()
}

fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn contains_word(source: &str, word: &str) -> bool {
    let mut search_start = 0;

    while let Some(pos) = source[search_start..].find(word) {
        let abs = search_start + pos;
        let before_ok = abs == 0 || !is_ident_char(source[..abs].chars().last().unwrap());
        let after = abs + word.len();
        let after_ok = after >= source.len() || !is_ident_char(source[after..].chars().next().unwrap());

        if before_ok && after_ok {
            return true;
        }

        search_start = abs + word.len();
    }

    false
}

fn strip_comments(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        output.push('\n');
                        break;
                    }
                }
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if c == '\n' {
                        output.push('\n');
                    }
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
                output.push(' ');
            }
            _ => output.push(c),
        }
    }

    output
}
