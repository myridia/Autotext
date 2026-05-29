#![windows_subsystem = "windows"]
use gtk::prelude::*;
use gtk::{ Button, Dialog, ApplicationWindow, Builder, AboutDialog, MessageDialog, ResponseType};
use gio::prelude::*;
use glib::clone;
use std::env::args;
use mylib::*;
use std::env;
use std::fs;
use gdk_pixbuf::Pixbuf;

fn build_ui(app: &gtk::Application)
{
  let resources_bytes = include_bytes!("../resources/resources.gresource");
  let resource_data = glib::Bytes::from(&resources_bytes[..]);
  let res = gio::Resource::from_data(&resource_data).unwrap();
  gio::resources_register(&res);
  let builder = Builder::from_resource("/org/calantas/autotext/main_window.glade");
  let window: ApplicationWindow = builder.get_object("Autotext").expect("Couldn't get window");
  window.set_application(Some(app));

  let cat_model = create_and_fill_model_cat();
  let category: gtk::TreeView = builder.get_object("category").unwrap();
  let _category: &'static str = "Category";
  append_column(&category, 0, _category, &cat_model,"0");
  category.set_model(Some(&cat_model));



   builder.connect_signals(|_builder, handler_name|
   {
     match handler_name
     {
       "save_text" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("save_text");
         let subcategory: gtk::TreeView = _builder.get_object("subcategory").unwrap();
         let selection = subcategory.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let text_view: gtk::TextView = _builder.get_object("text_view").unwrap();
           let textview = get_textview(&text_view);
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
           let _e = update_subcategory(&id, &textview);
           println!("subcategory id: {:?}",id);
         }
         None
       } )),

       "copy_text" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("copy_text");
         let text_view: gtk::TextView = _builder.get_object("text_view").unwrap();
         let textview = get_textview(&text_view);
         let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
         clipboard.set_text(&textview);
         None
       } )),


       "delete_category" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("delete category");
         let category: gtk::TreeView = _builder.get_object("category").unwrap();
         let selection = category.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
           println!("id: {:?}",id);
           let _e = delete_category(&id);

           let cat_model = create_and_fill_model_cat();
           let _category: &'static str = "Category";
           append_column(&category, 0, _category, &cat_model,"0");
           category.set_model(Some(&cat_model));
         }
         None
       } )),


       "add_category" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("add category");
         let _x = add_category();
         let cat_model = create_and_fill_model_cat();
         let category: gtk::TreeView = _builder.get_object("category").unwrap();
         let _category: &'static str = "Category";
         append_column(&category, 0, _category, &cat_model,"0");
         category.set_model(Some(&cat_model));
         None
       } )),


       "add_subcategory" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("add new subcategory");
         let category: gtk::TreeView = _builder.get_object("category").unwrap();
         let selection = category.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");

           let text_view: gtk::TextView = _builder.get_object("text_view").unwrap();
	   let textview = get_textview(&text_view);

           let _e = add_subcategory(&id, &textview);
           let subcat_model = create_and_fill_model_subcat(&id);
           let subcategory: gtk::TreeView = _builder.get_object("subcategory").unwrap();
           let _subcategory: &'static str = "Subcategory";
           append_column(&subcategory, 0, _subcategory, &subcat_model, &id);
           subcategory.set_model(Some(&subcat_model));
         }
         None
       } )),

       "delete_subcategory" => Box::new(clone!(@strong _builder =>  move |_|
       {
         println!("delete subcategory");

         let subcategory: gtk::TreeView = _builder.get_object("subcategory").unwrap();
         let selection = subcategory.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
           println!("id: {:?}",id);
           let cat_id = delete_subcategory(&id);

           println!("id: {:?}",cat_id);
           let cat_model = create_and_fill_model_subcat(&cat_id);
           let _subcategory: &'static str = "Subcategory";
           append_column(&subcategory, 0, _subcategory, &cat_model,"0");
           subcategory.set_model(Some(&cat_model));
         }
         
         None
       } )),



       "select_cat" => Box::new(clone!(@strong _builder =>  move |_|
       {
         let category: gtk::TreeView = _builder.get_object("category").unwrap();
         let selection = category.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
           let subcat_model = create_and_fill_model_subcat(&id);
           let subcategory: gtk::TreeView = _builder.get_object("subcategory").unwrap();
           let _subcategory: &'static str = "Subcategory";
           append_column(&subcategory, 0, _subcategory, &subcat_model, &id);
           subcategory.set_model(Some(&subcat_model));
         }
        //println!("click");
        None
       } )),

       "select_subcat" => Box::new(clone!(@strong _builder =>  move |_|
       {
         let subcategory: gtk::TreeView = _builder.get_object("subcategory").unwrap();
         let selection = subcategory.get_selection();
         if let Some((model, iter)) = selection.get_selected()
         {
           let id = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
           let content = get_subcategory_by_id( &id );
           let text_view: gtk::TextView = _builder.get_object("text_view").unwrap();
           text_view.get_buffer().expect("Couldn't get window").set_text(&content);
           let clipboard = gtk::Clipboard::get(&gdk::SELECTION_CLIPBOARD);
           clipboard.set_text(&content);
         }
        None
       } )),

       "close_app" => Box::new(clone!(@strong _builder =>  move |_| {

       let window: ApplicationWindow = _builder.get_object("Autotext").expect("Couldn't get window");
       window.close();
       None
       } )),

       "on_databases" => Box::new(clone!(@strong _builder =>  move |_| {
          let builder = Builder::from_resource("/org/calantas/autotext/main_window.glade");
          let p: Dialog = builder.get_object("database_dialog").expect("...cannot open database dialog");
          let password = get_password();
          let input_password: gtk::Entry = builder.get_object("input_password").unwrap();
          let check_save_password: gtk::CheckButton = builder.get_object("check_save_password").unwrap();
          input_password.set_text(&password);
          input_password.set_visibility(false);
          input_password.set_invisible_char(Some('*'));


          let db_model = get_databases();

          //println!("{:?}",databases);


          let databases: gtk::TreeView = builder.get_object("databases").unwrap();
          let _databases: &'static str = "Select a Databases";
          append_column(&databases, 0, _databases, &db_model,"0");
          databases.set_model(Some(&db_model));


          p.show_all();    



          let btn: Button = builder.get_object("btn_close").expect("Cant get button");
          btn.connect_clicked(clone!(@strong p =>  move |_| {
            p.close();
          }));


          let btn: Button = builder.get_object("btn_save").expect("Cant get button");
          btn.connect_clicked(clone!(@strong p =>  move |_| {
            let _password = input_password.get_text();
            let _save_password = check_save_password.get_active();
            println!("action:save | password: {0} | save_password: {1}",_password,_save_password);            
          }));


          let btn: Button = builder.get_object("btn_import").expect("Cant get button import");
            btn.connect_clicked(clone!(@strong p =>  move |_| {
            println!("{:?}","import".to_string());            
          }));

          let btn: Button = builder.get_object("btn_add").expect("Cant get button add");
            btn.connect_clicked(clone!(@strong p =>  move |_| {
            println!("{:?}","add".to_string());            
          }));

          let btn: Button = builder.get_object("btn_delete").expect("Cant get button delete");
            btn.connect_clicked(clone!(@strong p =>  move |_| {
            println!("{:?}","delete".to_string());            
          }));


         builder.connect_signals(|builder, h|
         {
           
           match h
           {

           "select_db" => Box::new(clone!(@strong builder =>  move |_|
             {
               println!("select_db");
               let db: gtk::TreeView = builder.get_object("databases").unwrap();
               let selection = db.get_selection();
               if let Some((model, iter)) = selection.get_selected()
               {
                 let id            = model.get_value(&iter, 1).get::<String>().expect("y").expect("x");
                 let password      = get_password_by_id(&id);
                 let save_password = get_save_password_by_id(&id);


                 println!("Id: {0} | Password: {1} | Save Password: {2}",id, password, save_password);
                 let input_password: gtk::Entry = builder.get_object("input_password").unwrap();
                 input_password.set_text(&password);
                 input_password.set_visibility(false);
                 input_password.set_invisible_char(Some('*'));

                 let check_save_password: gtk::CheckButton = builder.get_object("check_save_password").unwrap();
                 if save_password == "1"
                 {
                   check_save_password.set_active(true);
                 }
                 else
                 {
                   check_save_password.set_active(false);
                 }

               }


               None
             })),

            _ => Box::new(|_| {None})
           }
           
            // _ => Box::new(|_| {None})
         });



        None
       } )),



       "about" => Box::new(clone!(@strong _builder =>  move |_| {
         let logo = Pixbuf::from_file_at_scale("resources/Circle-A.svg",200,200,false).unwrap();
         let filename = "LICENSE.txt";
         let license = fs::read_to_string(filename).expect("Something went wrong reading the file");

         let a = AboutDialog::new();
         a.set_authors(&["veto@myridia.com"]);
         a.set_website_label(Some("http://autotext.calantas.org"));
         a.set_website(Some("https://autotext.calantas.org"));
         a.set_authors(&["Veto veto@myridia.com"]);
 	 a.set_title("About Autotext");
	 a.set_program_name("Autotext");
	 a.set_version(Some("1.0.0"));
         a.set_license(Some(&license));
         a.set_logo(Some(&logo));

         a.connect_response(clone!(@weak a => move |_a, _response| {
            _a.close();
         }));

         a.show_all();

       None
       } )),

       _ => Box::new(|_| {None})
      }
    });
  //foo();
  window.show_all();
}


fn main()
{
  let app = gtk::Application::new(Some("org.calanatas.autotext"), Default::default()).expect("init failed...");
  app.connect_activate(|app| {
     build_ui(app);             
     test();
  });

  app.run(&args().collect::<Vec<_>>());
}
