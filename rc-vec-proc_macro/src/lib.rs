use proc_macro::*;

#[proc_macro_attribute]
pub fn rc_impl_gen_arc_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    if let Some(attr) = attr.into_iter().next() {
        let err = Ident::new("invalid", attr.span());
        return TokenTree::from(err).into();
    }

    item.clone()
        .into_iter()
        .chain(rc_impl_gen_arc_impl_inner(item))
        .collect()
}

fn rc_impl_gen_arc_impl_inner(input: TokenStream) -> TokenStream {
    input.into_iter()
        .map(|tt| match tt {
            TokenTree::Group(group) => {
                let new = rc_impl_gen_arc_impl_inner(group.stream());
                let mut new_g = Group::new(group.delimiter(), new);
                new_g.set_span(group.span());
                new_g.into()
            },
            TokenTree::Ident(ident) => {
                let new = ident.to_string().replace("Rc", "Arc");
                Ident::new(&new, ident.span()).into()
            },
            //TokenTree::Literal(lit) => {
            //    let new = lit.to_string().replace("Rc", "Arc");
            //    let mut new_lit = Literal::from_str(&new).unwrap();
            //    new_lit.set_span(lit.span());
            //    new_lit.into()
            //},
            _ => tt,
        })
        .collect()
}
