use topcoat::{
    Result,
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::Cx,
    font::{Font, RouterBuilderFontExt, fontsource::fontsource_font},
    icon::{icon, iconify::iconify_icon},
    router::{RouterBuilder, Slot, layout, module_router, page},
    runtime::RuntimeLayer,
    tailwind,
    view::{View, attributes, view},
};

use crate::components::card::{
    card, card_description, card_header, card_title,
};

const GEIST: Font = fontsource_font!(GEIST, weight: [400, 700], style: Normal);

pub fn router() -> RouterBuilder {
    module_router!()
        .font(GEIST)
        .assets(AssetBundle::load().unwrap())
        .layer(RuntimeLayer::new())
}

#[layout]
async fn root(cx: &Cx, slot: Slot<'_>) -> Result<impl View> {
    Ok(view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>"web"</title>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::font::link(font: GEIST)
                topcoat::runtime::script()
                topcoat::dev::script()
            </head>
            <body>(slot)</body>
        </html>
    })
}

#[page]
async fn home(cx: &Cx) -> Result<impl View> {
    Ok(view! {
        <main class="mx-auto max-w-2xl px-4 py-16">
            card(
                card_header(
                    card_title(
                        icon(
                            data: iconify_icon!("lucide:sparkles"),
                            attrs: attributes! { class="inline-block" }
                        )
                        " "
                        "web"
                    )
                    card_description(
                        "Edit "
                        <code>"src/app.rs"</code>
                        " and save to see your changes."
                    )
                )
            )
        </main>
    })
}
