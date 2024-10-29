#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

//const TAILWIND_URL: Asset = asset!("assets/tailwind.css");

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting app");
    launch(App);
}

#[component]
fn App() -> Element {
    // Build cool things ✌️

    rsx! {
        div {
            class: "container",
            div {
                class: "grid grid-cols-4",
                h1 {
                    "字幕翻译"
                },

                input {
                    r#type: "file",
                }
            }
            div {

                table {
                    thead {
                        tr {
                            th {"index"},
                            th {"开始时间"},
                            th {"结束时间"},
                            th {"原文"},
                            th {"译文(可编辑)"},
                        }
                    },
                    tbody {
                        tr{
                            th{"0"},
                            td {"00:00:04"},
                            td {"00:00:04"},
                            th{"MUSIC"},
                            th{input {"音乐"}},
                        },
                    }
                }

            }
        }
    }
}
