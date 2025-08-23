//! Generate C code from the `codegen` output.
//!
//! This is a budget replacement for cargo-expand and a more project-specific version of cbindgen.
//!
//! Takes one input which is the filename of the destination C source.
//!
//! See Jelal's README for more information.
// TODO test on environments without rustfmt.

use std::process::Command;

use codegen::{
    resolve_type::TypeResolver,
    util::{is_ident, lit_str_expr, name_value_str, write_output},
    C_FEATURE, FILES_PREFIX, LIB_NAME, OUTPUT,
};
use quote::{format_ident, ToTokens};
use syn::{visit::*, Ident, Item};

fn main() {
    let dest = std::env::args()
        .nth(1)
        .expect("give the destination filename as input");

    println!("run from the root of this binary's project");

    // path is the same as module in this crate, just remove the extension
    let module_path = {
        let mut split = OUTPUT.split('/').collect::<Vec<_>>();
        split.last_mut().map(|i| {
            if i.ends_with(".rs") {
                *i = &i[..i.len() - 3];
            }
            i
        });
        split
            .iter()
            .map(|i| format_ident!("{}", i))
            .collect::<Vec<_>>()
    };

    let mut expand = expand();
    if !module_select(&mut expand.items, &module_path) {
        panic!("failed to find {:?} path", module_path);
    }

    let mut cffi = CFfi::default();
    cffi.visit_file(&expand);

    write_output(&dest, cffi.generate_content()).unwrap();
    println!("wrote: {:?}", &dest);
}

/// Run expand on the crate or throw error.
fn expand() -> syn::File {
    let features_flag = format!("--features={}", C_FEATURE);

    // TODO make more sense than this:
    let parent_dir = std::path::Path::new(FILES_PREFIX)
        .parent()
        .unwrap()
        .canonicalize()
        .unwrap();

    let mut cmd = Command::new("rustup");
    cmd.args([
        "run",
        "nightly",
        "cargo",
        "rustc",
        &features_flag,
        "--",
        "-Zunpretty=expanded",
    ])
    .current_dir(&parent_dir);

    let cmd_str = cmd
        .get_args()
        .fold(cmd.get_program().to_str().unwrap().to_owned(), |acc, i| {
            acc + " " + i.to_str().unwrap()
        });

    let stdout = match cmd.output() {
        Err(e) => {
            eprintln!("failed to run `{}`", cmd_str);
            panic!("{:?}", e);
        }
        Ok(output) => String::from_utf8(output.stdout).expect("invalid Unicode output of expand"),
    };

    syn::parse_file(&stdout).unwrap()
}

/// Retain a module marked by the path if exists (determines the return value), and delete the rest.
///
/// Returns true if the path was found and false otherwise.
fn module_select<'src, 'idents>(items: &'src mut Vec<Item>, module_path: &'idents [Ident]) -> bool {
    for ident in module_path {
        let Some(module) = items.iter_mut().find_map(|i| match i {
            Item::Mod(item_mod) if item_mod.ident.to_string() == ident.to_string() => {
                Some(item_mod)
            }
            _ => None,
        }) else {
            std::mem::take(items); // empty it since module was not found
            return false;
        };

        if let Some((_, mod_items)) = std::mem::take(&mut module.content) {
            *items = mod_items;
        } else {
            std::mem::take(items); // unlikely to success after this but still cannot be sure
        }
    }

    true
}

#[derive(Default)]
struct CFfi {
    pub type_resolver: TypeResolver,
    pub typedefs: String,
    pub structs: String,
    pub statics: String,
    pub fns: String,
}

impl CFfi {
    /// Create a final C source from the information available.
    pub fn generate_content(&self) -> String {
        format!(
            // TODO read the tags from Cargo.toml
            "\
             /**\n\
              * Automatically created using `codegen` internal crate.\n\
              *\n\
              * @repository https://crates.io/crate/jelal\n\
              * @license Licensed dually under MIT or Apache-2.0 (see the repository for more)\n\
              */\n\
              \n\
              #ifndef {pragma_marker}\n\
              #define {pragma_marker}\n\
              \n\
              #include <stdint.h>\n\
              #include <stdbool.h>\n\
              \n\
              {typedefs}\
              {structs}\
              #ifdef __cplusplus\n\
              extern \"C\" {{\n\
              #endif // __cplusplus\n\
              \n\
              {consts}\
              {fns}\
              #ifdef __cplusplus\n\
              }} // extern \"C\"\n\
              #endif // __cplusplus\n\
              \n\
              #endif // {pragma_marker}\
            ",
            pragma_marker = format!("{}_H", LIB_NAME.to_ascii_uppercase()),
            typedefs = self.typedefs,
            structs = self.structs,
            consts = self.statics,
            fns = self.fns,
        )
    }

    // TODO impl using traits
    /// Only select public items.
    fn is_acceptable_vis(vis: &syn::Visibility) -> bool {
        matches!(vis, syn::Visibility::Public(_))
    }

    /// Select the supported function signatures.
    fn is_acceptable_abi(abi: &Option<syn::Abi>) -> bool {
        abi.as_ref()
            .is_some_and(|i| i.name.as_ref().is_some_and(|i| i.value() == "C"))
    }

    /// Return the `doc` attribute if available and a literal string.
    ///
    /// Has a trailing "\n" if a valid line.
    ///
    /// This function only takes the first doc that is a literal string. It's suggested to collapse
    /// the documents using `expand`-like commands or `collapse-doc` runs or
    /// [`codegen::util::collapse_docs`].  Since the input comes directly from the `codegen` binary,
    /// this is already the case.
    fn doc(attrs: &Vec<syn::Attribute>) -> String {
        let Some(str_doc) = attrs
            .iter()
            .find_map(|i| name_value_str(i, "doc"))
            .map(|i| i.value())
        else {
            return Default::default();
        };
        let c_doc = format!(
            "/**\n{} */\n",
            str_doc
                .split('\n')
                .map(|i| format!(" *{}\n", i))
                .collect::<String>()
        );
        c_doc
    }

    /// Given a type, will resolve it to a C primitive if possible.
    ///
    /// The resulting type will have an extra space for formatting purposes.
    fn resolve_ctype(ty: &syn::Type) -> String {
        let mut results = match ty {
            syn::Type::Reference(v) => {
                format!(
                    "{}*{}",
                    Self::resolve_ctype(&v.elem),
                    match v.mutability {
                        Some(_) => "",
                        None => "const",
                    }
                )
            }
            // // TODO make the length available to C
            syn::Type::Slice(v) => format!("{}*const", Self::resolve_ctype(&v.elem)),
            syn::Type::Array(v) => format!("{}*const", Self::resolve_ctype(&v.elem)),
            syn::Type::Path(type_path)
                if type_path.qself.is_none() && type_path.path.require_ident().is_ok() =>
            {
                let ty_str = type_path.to_token_stream().to_string();
                match ty_str.as_str() {
                    "bool" => "bool",
                    "char" => "uint32_t",
                    "u8" => "uint8_t",
                    "u16" => "uint16_t",
                    "u32" => "uint32_t",
                    "u64" => "uint64_t",
                    "usize" => "uintptr_t",
                    "i8" => "int8_t",
                    "i16" => "int16_t",
                    "i32" => "int32_t",
                    "i64" => "int64_t",
                    "isize" => "intptr_t",
                    "f32" => "float",
                    "f64" => "double",
                    _ => &ty_str,
                }
                .to_owned()
            }
            syn::Type::Path(type_path) => panic!(
                "unacceptable path was passed to resolve function \
                 (import with `use` and make accessable to other modules instead): `{}`",
                type_path.to_token_stream().to_string()
            ),
            _ => panic!(
                "unacceptable type was passed to resolve function: `{}`",
                ty.to_token_stream().to_string()
            ),
        };
        if results.chars().last() != Some('*') {
            results.push(' ');
        }
        results
    }
}

/// Trusts that the output is from `codegen` binary.
impl<'a> Visit<'a> for CFfi {
    fn visit_file(&mut self, i: &syn::File) {
        self.type_resolver.visit_file(i);
        visit_file(self, i);
    }

    fn visit_item_static(&mut self, i: &'a syn::ItemStatic) {
        if !Self::is_acceptable_vis(&i.vis) {
            return;
        }

        // make sure it's no_mangle or exported
        // Since this is this edition of Rust, `unsafe` is required.
        let unsafes = i
            .attrs
            .iter()
            .rev() // reverse to read the last as the first valid response
            .find_map(|i| match i.meta.require_list() {
                Ok(list) if list.path.is_ident("unsafe") => Some(list),
                _ => None,
            });

        // try to read export_name or no_mangle and if none, just break the process
        let Some(export_name) = unsafes
            .clone()
            .iter()
            .find_map(|attr| match attr.parse_args::<syn::MetaNameValue>() {
                Ok(kv) if kv.path.is_ident("export_name") => {
                    lit_str_expr(&kv.value).map(|i| i.value())
                }
                _ => None,
            })
            .or_else(|| {
                unsafes
                    .iter()
                    .find_map(|attr| match attr.parse_args::<Ident>() {
                        Ok(ident) if ident.to_string() == "no_mangle" => Some(i.ident.to_string()),
                        _ => None,
                    })
            })
        else {
            return;
        };

        self.statics.push_str(&format!(
            "{}\
             extern {} {}{};\n\
             \n\
            ",
            Self::doc(&i.attrs),
            if i.mutability.to_token_stream().is_empty() {
                "const"
            } else {
                ""
            },
            Self::resolve_ctype(&i.ty),
            export_name,
        ));

        visit_item_static(self, i);
    }

    fn visit_item_fn(&mut self, i: &'a syn::ItemFn) {
        if !(Self::is_acceptable_vis(&i.vis) && Self::is_acceptable_abi(&i.sig.abi)) {
            return;
        }

        let ret = match &i.sig.output {
            syn::ReturnType::Default => "void ".to_owned(),
            syn::ReturnType::Type(_, ty) => Self::resolve_ctype(&ty),
        };

        let params = i
            .sig
            .inputs
            .iter()
            .map(|i| match i {
                syn::FnArg::Receiver(_) => panic!("method defined as global function"),
                syn::FnArg::Typed(pat_type) => match &*pat_type.pat {
                    syn::Pat::Ident(pat_ident) if pat_ident.by_ref.is_none() => {
                        let ident = pat_ident.ident.to_string();
                        format!(
                            "{}{}{}",
                            match &pat_ident.mutability {
                                Some(_) => "",
                                None => "const ",
                            },
                            Self::resolve_ctype(&pat_type.ty),
                            if ident == "this" { "self" } else { &ident } // `this` is reserved in C
                        )
                    }
                    _ => panic!("only owned ident pat types are supported as of now"),
                },
            })
            .reduce(|acc, i| acc + ", " + &i)
            .unwrap_or_default();

        self.fns.push_str(&format!(
            "{}\
             {}{}({});
             \n\
            ",
            Self::doc(&i.attrs),
            ret,
            i.sig.ident.to_string(),
            params,
        ));

        visit_item_fn(self, i);
    }

    fn visit_item_type(&mut self, i: &'a syn::ItemType) {
        // TODO instead of checking for it being an ident, the original sift of codegen must not
        // allow for tuples to be created
        if !(Self::is_acceptable_vis(&i.vis) && is_ident(&i.ty)) {
            return;
        }

        self.typedefs.push_str(&format!(
            "\
             {}\
             typedef {}{};\n\
             \n\
            ",
            Self::doc(&i.attrs),
            Self::resolve_ctype(&i.ty),
            i.ident.to_string(),
        ));

        visit_item_type(self, i);
    }

    fn visit_item_struct(&mut self, i: &'a syn::ItemStruct) {
        if !Self::is_acceptable_vis(&i.vis) {
            return;
        }

        let ident_str = i.ident.to_string();
        let doc = Self::doc(&i.attrs);

        // There are two cases for a struct, it's either a "dissolvable" type meaning it's like a
        // `transparent` and can be equal to a `repr(int)` in that case adding it as a simple
        // `typedef` will do. `TypeResolver` will determine if this is the case, in other cases,
        // the struct will be added as expected.
        //
        // Note that only `TypeResolver` from Rust properties (like `transparent` or other systems
        // like being `transmute`-able) can do this and this should not be up to C to decide, from
        // the C code.
        //
        // In this crate, any alias returned from a `TypeResolver` is guaranteed to work.
        if let Some(alias) = self.type_resolver.repr_alias(&ident_str) {
            self.typedefs.push_str(&format!(
                "\
                 {}\
                 typedef {} {};\n\
                 \n\
                ",
                doc, alias, ident_str,
            ));
        } else {
            let fields = i
                .fields
                .iter()
                .enumerate()
                .map(|(i, field)| {
                    let ident = field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or(format!("m{}", i));
                    let ty = Self::resolve_ctype(&field.ty);
                    format!("  {}{};\n", ty, ident)
                })
                .collect::<String>();
            self.structs.push_str(&format!(
                "\
                 {}\
                 typedef struct {2} {{\n\
                 {}\
                 }} {2};\n\
                 \n\
                ",
                doc, fields, ident_str,
            ));
        }

        visit_item_struct(self, i);
    }
}
