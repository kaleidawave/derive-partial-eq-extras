use proc_macro::TokenStream;
use syn_helpers::{
    derive_trait,
    proc_macro2::{Ident, Span},
    quote,
    syn::{
        parse_macro_input, parse_quote, BinOp, DeriveInput, Expr, ExprBinary, ExprLit, Lit,
        LitBool, Stmt, Token, Type,
    },
    CommaSeparatedList, Constructable, Field, FieldMut, Fields, HasAttributes, Trait, TraitItem,
};

const LEFT_NAME_POSTFIX: &str = "_left";
const RIGHT_NAME_POSTFIX: &str = "_right";

#[proc_macro_derive(
    PartialEqExtras,
    attributes(partial_eq_ignore_types, partial_eq_ignore)
)]
pub fn partial_eq_extras(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let eq_item = TraitItem::new_method(
        Ident::new("eq", Span::call_site()),
        None,
        syn_helpers::TypeOfSelf::Reference,
        vec![parse_quote!(other: &Self)],
        Some(parse_quote!(bool)),
        |item| {
            let attributes = item.structure.get_attributes();

            let ignored_types: Vec<Type> = attributes
                .iter()
                .filter(|attr| attr.path().is_ident(IGNORE_TYPES))
                .flat_map(|attr| {
                    attr.parse_args::<CommaSeparatedList<Type>>()
                        .unwrap()
                        .into_iter()
                })
                .collect();

            match item.structure {
                syn_helpers::Structure::Struct(r#struct) => {
                    let expr =
                        build_comparison_for_fields(r#struct.get_fields_mut(), &ignored_types);

                    let left_patterns = r#struct.get_fields().to_pattern_with_config(
                        r#struct.get_constructor_path(),
                        syn_helpers::TypeOfSelf::Reference,
                        LEFT_NAME_POSTFIX,
                    );
                    let right_patterns = r#struct.get_fields().to_pattern_with_config(
                        r#struct.get_constructor_path(),
                        syn_helpers::TypeOfSelf::Reference,
                        RIGHT_NAME_POSTFIX,
                    );
                    let declaration = parse_quote! {
                        let (#left_patterns, #right_patterns) = (self, other);
                    };

                    Ok(vec![declaration, Stmt::Expr(expr, None)])
                }
                syn_helpers::Structure::Enum(r#enum) => {
                    let branches = r#enum.get_variants_mut().iter_mut().map(|variant| {
                        let expr =
                            build_comparison_for_fields(variant.get_fields_mut(), &ignored_types);

                        let left_patterns = variant.get_fields().to_pattern_with_config(
                            variant.get_constructor_path(),
                            syn_helpers::TypeOfSelf::Reference,
                            LEFT_NAME_POSTFIX,
                        );
                        let right_patterns = variant.get_fields().to_pattern_with_config(
                            variant.get_constructor_path(),
                            syn_helpers::TypeOfSelf::Reference,
                            RIGHT_NAME_POSTFIX,
                        );
                        let token_stream = quote! { (#left_patterns, #right_patterns) => #expr };
                        token_stream
                    });
                    let match_stmt = parse_quote! {
                        match (self, other) {
                            #(#branches,)*
                            (_, _) => false
                        }
                    };
                    Ok(vec![match_stmt])
                }
            }
        },
    );

    // PartialEq trait
    let partial_eq_trait = Trait {
        name: parse_quote!(::std::cmp::PartialEq),
        generic_parameters: None,
        items: vec![eq_item],
    };

    let derive_trait = derive_trait(input, partial_eq_trait);
    derive_trait.into()
}

const IGNORE_TYPES: &str = "partial_eq_ignore_types";
const IGNORE_FIELD: &str = "partial_eq_ignore";

fn build_comparison_for_fields(fields: &mut Fields, ignored_types: &[Type]) -> Expr {
    let mut top = None::<Expr>;

    for mut field in fields.fields_iterator_mut() {
        if field
            .get_attributes()
            .iter()
            .any(|attr| attr.path().is_ident(IGNORE_FIELD))
        {
            continue;
        }

        let ignore_type_reference_and_on_tests =
            ignored_types.iter().any(|ty| field.get_type() == ty);

        if ignore_type_reference_and_on_tests {
            continue;
        }

        let lhs = field.get_reference_with_config(true, LEFT_NAME_POSTFIX);
        let rhs = field.get_reference_with_config(true, RIGHT_NAME_POSTFIX);

        let expr = Expr::Binary(ExprBinary {
            attrs: Vec::new(),
            left: Box::new(lhs),
            op: BinOp::Eq(Token!(==)(Span::call_site())),
            right: Box::new(rhs),
        });

        if let Some(old_top) = top {
            top = Some(Expr::Binary(ExprBinary {
                attrs: Vec::new(),
                left: Box::new(old_top),
                op: BinOp::And(Token!(&&)(Span::call_site())),
                right: Box::new(expr),
            }));
        } else {
            top = Some(expr);
        }
    }
    top.unwrap_or_else(|| {
        Expr::Lit(ExprLit {
            attrs: vec![],
            lit: Lit::Bool(LitBool {
                value: true,
                span: Span::call_site(),
            }),
        })
    })
}
