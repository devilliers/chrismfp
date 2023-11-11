use std::collections::HashMap;

use gloo::file::callbacks::FileReader;
use gloo::file::File;
use web_sys::{DragEvent, Event, FileList, HtmlInputElement};
use yew::html::TargetCast;
use yew::{html, Callback, Component, Context, Html};
mod mfp;

#[allow(dead_code)]
struct FileDetails {
    name: String,
    file_type: String,
    data: String,
    data_type: String,
}

pub enum Msg {
    Loaded(String, String, String, String),
    Files(Vec<File>),
}

pub struct App {
    readers: HashMap<String, FileReader>,
    files: Vec<FileDetails>,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            readers: HashMap::default(),
            files: Vec::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Loaded(file_name, file_type, data, data_type) => {
                self.files.push(FileDetails {
                    data,
                    data_type,
                    file_type,
                    name: file_name.clone(),
                });
                self.readers.remove(&file_name);
                true
            }
            Msg::Files(files) => {
                for file in files.into_iter() {
                    let file_name = file.name();
                    let file_type = file.raw_mime_type();

                    println!("{}", &file_name);
                    let task = {
                        let link = ctx.link().clone();
                        let file_name = file_name.clone();

                        gloo::file::callbacks::read_as_text(&file, move |res| {
                            let res_ok = res.expect("failed to read file");
                            let file_bytes = res_ok.as_bytes();
                            let process_file_type = {
                                if file_name.contains("Exercise") {
                                    "Steps"
                                } else if file_name.contains("Nutrition") {
                                    "Macros"
                                } else if file_name.contains("Measurement") {
                                    "Weight"
                                } else {
                                    ""
                                }
                            };
                            let processed_file = mfp::process(file_bytes, process_file_type);
                            link.send_message(Msg::Loaded(
                                file_name,
                                file_type,
                                processed_file,
                                process_file_type.to_string(),
                            ))
                        })
                    };
                    self.readers.insert(file_name, task);
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="wrapper">
                <p id="title">{ "🏋️ MyFitnessPal ➡️ Chris' Google Sheets 📈" }</p>
                <p>{"1. Export your data from MyFitnessPal (instructions "}<a href="https://support.myfitnesspal.com/hc/en-us/articles/360032273352-Data-Export-FAQs">{"here"}</a>{")"}</p>
                <p>{"2. Process the files below..."}</p>
                <p>{"3. Paste into Chris' spreadsheets (without formatting, or the text will be invisible!)"}</p>
                <label for="file-process">
                    <div
                        id="drop-container"
                        ondrop={ctx.link().callback(|event: DragEvent| {
                            event.prevent_default();
                            let files = event.data_transfer().unwrap().files();
                            Self::process_files(files)
                        })}
                        ondragover={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                        ondragenter={Callback::from(|event: DragEvent| {
                            event.prevent_default();
                        })}
                    >
                        <i class="fa fa-cloud-upload"></i>
                        <p>{"Drag and drop your files here, or click to choose files"}</p>
                    </div>
                </label>
                <input
                    id="file-process"
                    type="file"
                    accept=".csv"
                    multiple={true}
                    onchange={ctx.link().callback(move |e: Event| {
                        let input: HtmlInputElement = e.target_unchecked_into();
                        Self::process_files(input.files())
                    })}
                />
                <div id="preview-area">
                    { for self.files.iter().map(Self::view_file) }
                </div>
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <br />
                <p class="line"></p>
                <p>{ "Note - no data leaves your computer; all processing is done within your browser." }</p>
            </div>
        }
    }
}

impl App {
    fn view_file(file: &FileDetails) -> Html {
        html! {
            <div class="preview-tile">
                <p class="preview-name"><b>{ format!("{}", file.data_type) }</b>{format!(" ({})", file.name) }</p>
                <pre class="preview-media">{ format!("{}", &file.data) }</pre>
            </div>
        }
    }

    fn process_files(files: Option<FileList>) -> Msg {
        let mut result = Vec::new();

        if let Some(files) = files {
            let files = js_sys::try_iter(&files)
                .unwrap()
                .unwrap()
                .map(|v| web_sys::File::from(v.unwrap()))
                .map(File::from);
            result.extend(files);
        }
        Msg::Files(result)
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
