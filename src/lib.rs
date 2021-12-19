use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use std::error::Error;
use syn::{
    parse_macro_input, parse_quote, BinOp, DeriveInput, Expr, ExprBinary, ExprLit, Lit, LitBool,
    Stmt, Token, Type,
};
use syn_helpers::{
    build_implementation_over_structure, BuildPair, CommaSeparatedList, Field, Fields,
    PrefixAndPostfix, Trait, TraitMethod,
};

#[proc_macro_derive(
    PartialEqExtras,
    attributes(partial_eq_ignore_types, partial_eq_ignore)
)]
pub fn partial_eq_extras(input: TokenStream) -> TokenStream {
    let structure = parse_macro_input!(input as DeriveInput);

    // PartialEq trait
    let partial_eq_trait = Trait {
        name: parse_quote!(::std::cmp::PartialEq),
        generic_parameters: vec![],
        methods: vec![TraitMethod {
            method_name: Ident::new("eq", Span::call_site()),
            method_parameters: vec![parse_quote!(&self), parse_quote!(other: &Self)],
            method_generics: vec![],
            return_type: Some(parse_quote!(bool)),
            build_pair: BuildPair::Pair {
                statements_if_enums_do_not_match: vec![parse_quote!(return false;)],
                other_item_name: Ident::new("other", Span::call_site()),
            },
        }],
    };

    build_implementation_over_structure(
        &structure,
        partial_eq_trait,
        |_, _| Ok(PrefixAndPostfix::default()),
        |method_name, fields| {
            if method_name == "eq" {
                partial_eq_extras_impl(fields)
            } else {
                unreachable!();
            }
        },
    )
    .into()
}

const IGNORE_TYPES: &str = "partial_eq_ignore_types";
const IGNORE_FIELD: &str = "partial_eq_ignore";

fn partial_eq_extras_impl(fields: &mut Fields) -> Result<Vec<Stmt>, Box<dyn Error>> {
    let ignored_types: Vec<Type> = fields
        .get_structure()
        .all_attributes()
        .find_map(|attr| {
            if attr.path.is_ident(IGNORE_TYPES) {
                Some(
                    attr.parse_args::<CommaSeparatedList<Type>>()
                        .unwrap()
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .unwrap_or_default();

    let true_expr = Expr::Lit(ExprLit {
        attrs: vec![],
        lit: Lit::Bool(LitBool {
            value: true,
            span: Span::call_site(),
        }),
    });

    let expr = match fields {
        Fields::Named { fields, .. } => {
            let mut top = None::<Expr>;
            for field in fields {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path.is_ident(IGNORE_FIELD))
                {
                    continue;
                }
                if ignored_types.iter().any(|ty| ty == field.ty) {
                    continue;
                }
                let (lhs, rhs) = (field.get_reference(), field.get_other_reference());
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
            top.unwrap_or(true_expr.clone())
        }
        Fields::Unnamed { fields, .. } => {
            let mut top = None::<Expr>;
            for field in fields {
                if field
                    .attrs
                    .iter()
                    .any(|attr| attr.path.is_ident(IGNORE_FIELD))
                {
                    continue;
                }
                if ignored_types.iter().any(|ty| ty == field.ty) {
                    continue;
                }
                let (lhs, rhs) = (field.get_reference(), field.get_other_reference());
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
            top.unwrap_or(true_expr.clone())
        }
        Fields::Unit { .. } => true_expr,
    };
    Ok(vec![Stmt::Expr(expr)])
}
