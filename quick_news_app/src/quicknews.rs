use std::{borrow::Cow, iter::FromIterator};
use eframe::egui::{FontDefinitions,FontFamily,Color32,Label,Layout,Hyperlink,Separator,TopBottomPanel,menu,TextStyle,Button,Window,Key};
use serde::{Serialize,Deserialize};
use confy::load;
use std::default::Default;
use std::sync::mpsc::{Receiver, SyncSender};

pub const PADDING:f32 = 5.0;
const WHITE: Color32 = Color32::from_rgb(255, 255, 255);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);
const BLUE: Color32 = Color32::from_rgb(0, 96, 250);

//enum type for Msgs
pub enum Msg {
    ApiKeySet(String),
    Refresh,
}

//structure for configuration
#[derive(Serialize,Deserialize)]
pub struct QuickNewsConfig {
    pub dark_mode : bool,
    pub api_key : String
}

impl Default for QuickNewsConfig{
    fn default() -> Self {
        Self {
            dark_mode : Default::default(),
            api_key : String::new()
        }
    }
}

//sturcture for App components
pub struct QuickNews {
    pub articles : Vec<NewsCardData>,
    pub configure : QuickNewsConfig,
    pub api_key_init : bool,
    pub news_rx: Option<Receiver<NewsCardData>> ,
    pub app_tx : Option<SyncSender<Msg>>,
}

//structure for news card
pub struct NewsCardData {
    pub title : String,
    pub description : String,
    pub url : String
}

impl QuickNews {
    pub fn new() -> QuickNews {
        //initializing structure
        QuickNews {
            api_key_init : Default::default(),
            articles : vec![],
            configure : Default::default(),
            news_rx : None,
            app_tx : None
        }
    }
    
    //configuring fonts for App
    pub fn configure_fonts(&self, ctx: &eframe::egui::CtxRef) {
        let mut font_def = FontDefinitions::default();
        font_def.font_data.insert("Roboto".to_string(),
        Cow::Borrowed(include_bytes!("../../RobotoSlabRegular-w1GE3.ttf")));

        font_def.family_and_size.insert(eframe::egui::TextStyle::Heading,(FontFamily::Proportional, 35.));

        font_def.family_and_size.insert(eframe::egui::TextStyle::Body,(FontFamily::Proportional, 20.));

        font_def.fonts_for_family.get_mut(&FontFamily::Proportional).unwrap().insert(0, "Roboto".to_string());

        ctx.set_fonts(font_def);

    }

    //for rendering news article
    pub fn render_news_cards(&self,ui:&mut eframe::egui::Ui) {
        for a in &self.articles {
            //title
            ui.add_space(PADDING);
            let title = format!{"▶ {}",a.title};
            if self.configure.dark_mode  {
                ui.colored_label(WHITE,title);
            }else {
                ui.colored_label(BLACK,title);
            }
            
            //description
            ui.add_space(PADDING);
            let description = Label::new(&a.description).text_style(eframe::egui::TextStyle::Button);
            ui.add(description);
            //url
            ui.style_mut().visuals.hyperlink_color = BLUE;
            ui.add_space(PADDING);
            ui.with_layout(Layout::right_to_left(), |ui| {
                ui.add(Hyperlink::new(&a.url).text("read more ↗"))
            });
            ui.add_space(PADDING);
            ui.add(Separator::default());
        }
    }
    
    //for rendering top panel of App
    pub fn top_panel_rendering(&mut self, ctx: &eframe::egui::CtxRef,frame:&mut eframe::epi::Frame<'_>) {
        TopBottomPanel::top("top panel").show(ctx,|ui|{
            menu::bar(ui, |ui| {
                //logo
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add(Label::new("📰").text_style(TextStyle::Heading));
                });
                //action panel
                ui.with_layout(Layout::right_to_left(), |ui| {
                    //close button
                    let close_button = ui.add(Button::new("❌").text_style(TextStyle::Body));
                    if close_button.clicked() {
                        frame.quit();
                    }

                    //refresh button
                    let refresh_button = ui.add(Button::new("🔃").text_style(TextStyle::Body));
                    if refresh_button.clicked() {
                        //self.articles.clear();
                        if let Some(tx) = &self.app_tx{
                            tx.send(Msg::Refresh);
                        }
                    }
                    
                    //theme button
                    let theme_button = ui.add(Button::new({
                        if self.configure.dark_mode {
                            "🌞"
                        }else {
                            "🌙"
                        }
                    }).text_style(TextStyle::Body));
                    
                    if theme_button.clicked() {
                        self.configure.dark_mode = !self.configure.dark_mode;
                    }
                });
            });
            ui.add_space(5.);
        });
    }

    //preloading articles on App
    pub fn preload_articles(&mut self) {
        if let Some(rx) = &self.news_rx {
            match rx.try_recv() {
                Ok(news_data) => {
                    self.articles.push(news_data);
                },
                Err(e) => {
                    tracing::warn!("Error receiving data: {}", e);
                }
            }
        }
    }

    //rendering configuration prompt of App (for the first time run)
    pub fn config_rendering(&mut self, ctx: &eframe::egui::CtxRef){
        Window::new("Configuration").show(ctx,|ui| {
            ui.label("Enter API Key of newsapi.org");
            let input = ui.text_edit_singleline(&mut self.configure.api_key);
            if input.lost_focus() && ui.input().key_pressed(Key::Enter){
                // to get key and mode from configuration
                if let Err(e) = confy::store("quicknews", QuickNewsConfig{
                    dark_mode : self.configure.dark_mode,
                    api_key : self.configure.api_key.to_string()
                }){
                    tracing::error!("Failed saving app state : {}",e);
                }
                
                self.api_key_init=true;
                if let Some(tx) = &self.app_tx{
                    tx.send(Msg::ApiKeySet(self.configure.api_key.to_string()));
                }
                tracing::error!("api key set");
            }
            ui.label("If you don't have API key, get one from");
            ui.hyperlink("https://newsapi.org");
        });
    }
}
