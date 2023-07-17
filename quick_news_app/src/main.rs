mod quicknews;

use quicknews::{QuickNews,PADDING,QuickNewsConfig,NewsCardData,Msg};
use eframe::{epi::{App,NativeOptions}, run_native};
use eframe::egui::{containers::{CentralPanel,ScrollArea},Vec2,Ui,Color32,Label,Separator,TopBottomPanel,Hyperlink,TextStyle,Visuals};
use newsapi::{NewsAPI,Article};
use std::{sync::mpsc::{channel,sync_channel},thread};

const YELLOW: Color32 = Color32::from_rgb(250, 242, 51);
const RED: Color32 = Color32::from_rgb(255,0,0);

//implementing App trait for Quicknews struct
impl App for QuickNews {

    //primarely setting up App 
    fn setup(&mut self,ctx:&eframe::egui::CtxRef,frame:&mut eframe::epi::Frame<'_>, storage: Option<&dyn eframe::epi::Storage> ) {
        //to store configuration setting of App
        if let Some(storage) = storage {
            self.configure = eframe::epi::get_value(storage, "headlines").unwrap_or_default();
            self.api_key_init = !self.configure.api_key.is_empty();
        }

        let api_key = self.configure.api_key.to_string();

        //creating channel for sending and receiveing information for both API and App 
        let (mut news_tx,news_rx) = channel();
        let (app_tx,app_rx) = sync_channel(1);
        self.app_tx = Some(app_tx);
        self.news_rx = Some(news_rx);
    
        thread::spawn( move|| {
            // to check if api key is empty or not
            if !api_key.is_empty() {
                fetch_news(&api_key, &mut news_tx);
            }else {
                loop {
                    //matching receiving info from app 
                    match app_rx.recv() {
                        Ok(Msg::ApiKeySet(api_key)) => {
                            fetch_news(&api_key,&mut news_tx);
                        },
                        Ok(Msg::Refresh) => {
                            fetch_news(&api_key,&mut news_tx);
                        }
                        Err(e) => {
                            tracing::error!("failed receiving msg : {}", e)
                        }
                        
                    }
                }
            }
        });
        //configuring fonts
        self.configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::CtxRef, frame:&mut eframe::epi::Frame<'_>){

        ctx.request_repaint(); //asking egui to load news article 

        //configuring modes for App
        if self.configure.dark_mode {
            ctx.set_visuals(Visuals::dark());
        }else {
            ctx.set_visuals(Visuals::light());
        }
        
        //to check API key initialization and rendering the configuration
        if !self.api_key_init {
            self.config_rendering(ctx);
        }else {
            self.preload_articles();
            
            //calling fucntion to rendering top panel of the application
            self.top_panel_rendering(ctx,frame);

            //central panel rendering
            CentralPanel::default().show(ctx,|ui| {
                if self.articles.is_empty() {
                    ui.vertical_centered_justified(|ui| {
                        ui.heading("Loading...");
                    });
                }else {
                    header_rendering(&self, ui);
                    ScrollArea::auto_sized().show(ui, |ui| {
                        self.render_news_cards(ui);
                    });
                }

            });
            footer_rendering(ctx);

        }

    }
    
    //saves configuration
    fn save(&mut self, storage: &mut dyn eframe::epi::Storage) {
        eframe::epi::set_value(storage, "headlines", &self.configure);
    }

    //setting name of App
    fn name(&self) -> &str {
        "QuickNews"
    }
}

//fetcing news from API
fn fetch_news(api_key : &str , news_tx :&mut std::sync::mpsc::Sender<NewsCardData> ) {
    if let Ok(response) = NewsAPI::new(&api_key).fetch() {               
        let resp_articles: &Vec<Article> = response.articles();

        //iterating through articles 
        for a in resp_articles.iter(){
            //initializing news card data
            let news :NewsCardData = NewsCardData {
                title : a.title().to_string(),
                url : a.url().to_string(),
                description: a.desc().map(|s:&String| s.to_string()).unwrap_or("...".to_string())
            };
            if let Err(e) = news_tx.send(news){
                tracing::error!("Error sending news data : {}", e);
            }
        }
    }else {
        tracing::error!("news fetching failed")
    }
}

// function for rendering header of the App
fn header_rendering(quick : &QuickNews,ui: &mut Ui) {
    ui.vertical_centered(|ui| {
    ui.add(Label::new("QuickNews").text_color({
        if quick.configure.dark_mode {
            YELLOW
        }else {
            RED
        }
    }));
    });
    ui.add_space(PADDING);
    let sep = Separator::default().spacing(20.);
    ui.add(sep);

}

// function for rendering header of the App
fn footer_rendering(ctx: &eframe::egui::CtxRef ){
    TopBottomPanel::bottom("footer").show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(10.);
            ui.add(Label::new("API source : newsapi.org").monospace());
            ui.add(Label::new("Build with \"Rust\" and \"egui\"").monospace());
            ui.add(Hyperlink::new("https://naitik-makwana.netlify.app/").text("Developed by Naitik Makwana").text_style(TextStyle::Monospace));
            ui.add_space(10.);
        });
    });
}

fn main() {
    //for tracing error on real time
    tracing_subscriber::fmt::init();

    //initiating new App
    let app = QuickNews::new();

    //creating window for App
    let mut window_option =  NativeOptions::default();
    //setting dimensions of App
    window_option.initial_window_size = Some(Vec2::new(380. , 700.));

    //to run App on native system
    run_native(Box::new(app), window_option);
}
