use base64::{Engine as _, engine::general_purpose};
use blake2::digest::{Update, VariableOutput};
use blake2::Blake2bVar;
use cryptostream::write;
use glib::clone;
use glib::StaticType;
use glib::ToValue;
use gtk::prelude::{
    CellLayoutExt, CellRendererTextExt, GtkListStoreExtManual, ObjectExt, TextBufferExt,
    TextViewExt, TreeModelExt, TreeViewColumnExt,
};
use gtk::{CellRendererText, ListStore, TreeView, TreeViewColumn};
use openssl::symm::Cipher;
use regex::Regex;
use std::io::Write;
use std::path::PathBuf;

fn data_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    let mut path = PathBuf::from(home);
    path.push(".autotext");
    path
}

pub fn init_dbs() {
    let dir = data_dir();
    std::fs::create_dir_all(&dir).unwrap();

    let settings = dir.join("settings.db");
    if !settings.exists() {
        let conn = sqlite::open(&settings).unwrap();
        conn.execute(
            "CREATE TABLE databases (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                database TEXT DEFAULT '',
                password TEXT DEFAULT '',
                save_password INTEGER DEFAULT 0,
                [default] INTEGER DEFAULT 0
            )",
        )
        .unwrap();
    }

    let db = dir.join("database.db");
    if !db.exists() {
        let conn = sqlite::open(&db).unwrap();
        conn.execute(
            "CREATE TABLE categories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT DEFAULT '',
                sort INTEGER DEFAULT 0
            )",
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE subcategories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT DEFAULT '',
                category_id INTEGER DEFAULT 0,
                content TEXT DEFAULT '',
                sort INTEGER DEFAULT 0
            )",
        )
        .unwrap();
    }
}

fn settings_path() -> String {
    let mut p = data_dir();
    p.push("settings.db");
    p.to_string_lossy().to_string()
}

fn database_path() -> String {
    let mut p = data_dir();
    p.push("database.db");
    p.to_string_lossy().to_string()
}

pub fn test() {
    let t = "hello world".to_string();
    let pass = "".to_string();
    let ct = encrypt(t, pass.clone());
    let s = decrypt(ct, pass.clone());
    println!("{:?}", s.to_string());
}

pub fn encrypt(t: String, password: String) -> String {
    let mut hasher = Blake2bVar::new(16).unwrap();
    hasher.update(password.as_bytes());
    let mut _key = [0u8; 16];
    hasher.finalize_variable(&mut _key).unwrap();
    let src: &[u8] = &t.into_bytes();
    let key: Vec<_> = _key.to_vec();

    println!("Result: {:?}", key.len());
    let iv: Vec<_> = general_purpose::STANDARD.decode("dB0Ej+7zWZWTS5JUCldWMg==").unwrap();
    let cipher = Cipher::aes_128_cbc();
    let mut encrypted = Vec::new();
    let mut bytes_written = 0;
    {
        let mut encryptor = write::Encryptor::new(&mut encrypted, cipher, &key, &iv).unwrap();
        while bytes_written < src.len() {
            let write_bytes = encryptor.write(&src[bytes_written..]).unwrap();
            bytes_written += write_bytes;
        }
    }

    let ct = general_purpose::STANDARD.encode(encrypted);
    return ct;
}

pub fn decrypt(ct: String, password: String) -> String {
    let mut hasher = Blake2bVar::new(16).unwrap();
    hasher.update(password.as_bytes());
    let mut _key = [0u8; 16];
    hasher.finalize_variable(&mut _key).unwrap();
    let key: Vec<_> = _key.to_vec();
    let src: Vec<u8> = general_purpose::STANDARD.decode(ct).unwrap();
    let iv: Vec<_> = general_purpose::STANDARD.decode("dB0Ej+7zWZWTS5JUCldWMg==").unwrap();
    let mut decrypted = Vec::new();
    {
        let mut decryptor =
            write::Decryptor::new(&mut decrypted, Cipher::aes_128_cbc(), &key, &iv).unwrap();
        let mut bytes_decrypted = 0;
        while bytes_decrypted != src.len() {
            let write_count = decryptor.write(&src[bytes_decrypted..]).unwrap();
            bytes_decrypted += write_count;
        }
    }
    let t = String::from_utf8_lossy(&decrypted);
    return t.into_owned(); // memory is allocated for a new stri
}

pub fn get_password() -> String {
    let mut password = "".to_string();
    let connection = sqlite::open(&settings_path()).unwrap();
    let statement = connection
        .prepare("SELECT password FROM databases WHERE [default] = 1 ")
        .unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        password = row.read::<&str, _>(0).to_string();
    }
    return password;
}

pub fn get_password_by_id(id: &String) -> String {
    let mut password = "".to_string();
    let connection = sqlite::open(&settings_path()).unwrap();
    let sql = format!("SELECT password FROM databases WHERE id = {} LIMIT 1", id);
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        password = row.read::<&str, _>(0).to_string();
    }
    return password;
}

pub fn get_save_password_by_id(id: &String) -> String {
    let mut save_password = "".to_string();
    let connection = sqlite::open(&settings_path()).unwrap();
    let sql = format!(
        "SELECT save_password FROM databases WHERE id = {} LIMIT 1",
        id
    );
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        save_password = row.read::<i64, _>(0).to_string();
    }
    return save_password;
}

pub fn get_databases() -> ListStore {
    let model = ListStore::new(&[String::static_type(), String::static_type()]);
    let connection = sqlite::open(&settings_path()).unwrap();
    let statement = connection.prepare("SELECT [database],cast([id] AS text),[password],cast([default] as text) FROM [databases] ORDER BY [id]").unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        let name = row.read::<&str, _>(0).to_string();
        let id = row.read::<&str, _>(1).to_string();
        model.insert_with_values(None, &[(0, &name), (1, &id)]);
    }
    model
}

pub fn create_and_fill_model_cat() -> ListStore {
    let model = ListStore::new(&[String::static_type(), String::static_type()]);
    let connection = sqlite::open(&database_path()).unwrap();
    let statement = connection
        .prepare("SELECT name, cast(id as text) FROM categories ORDER BY sort, name ")
        .unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        let name = row.read::<&str, _>(0).to_string();
        let id = row.read::<&str, _>(1).to_string();
        model.insert_with_values(None, &[(0, &name), (1, &id)]);
    }
    model
}

pub fn create_and_fill_model_subcat(id: &String) -> ListStore {
    let model = ListStore::new(&[String::static_type(), String::static_type()]);
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!("SELECT name, cast(id as text) FROM subcategories WHERE category_id = {} ORDER BY sort, name", id );
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        let name = row.read::<&str, _>(0).to_string();
        let id = row.read::<&str, _>(1).to_string();
        model.insert_with_values(None, &[(0, &name), (1, &id)]);
    }
    model
}

pub fn get_subcategory_by_id(id: &String) -> String {
    let mut ret = "".to_string();
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!(
        "SELECT content FROM subcategories WHERE id = {} LIMIT 1",
        id
    );
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        let content = row.read::<&str, _>(0).to_string();
        ret = content;
    }
    return ret;
}

pub fn append_column(
    tree: &TreeView,
    id: i32,
    title: &'static str,
    store: &ListStore,
    cat_id: &str,
) {
    use gtk::prelude::TreeViewExt;
    let _cat_id = string_to_static_str(cat_id.to_string());
    let col = TreeViewColumn::new();
    let cell = CellRendererText::new();
    let _e = cell.set_property("editable", true);

    cell.connect_edited(
        clone!(@weak store => move |_cellrenderertext, path, new_text|
        {
         if let Some(iter) = store.iter(&path)
         {
           store.set_value(&iter, 0, &new_text.to_value());
         }

          if title == "Category"
          {
            let _x = update_category_name(new_text.to_string(), path.to_string() );
          }
          else if  title == "Subcategory"
          {
            let _x = update_subcategory_name(new_text.to_string(), path.to_string(), &_cat_id);
           }
         }),
    );

    col.set_title(title);
    CellLayoutExt::pack_start(&col, &cell, true);
    CellLayoutExt::add_attribute(&col, &cell, "text", id);
    let ocol = tree.n_columns();
    if ocol > 0 {
        let n = tree.column(0).unwrap();
        tree.remove_column(&n);
    }

    tree.append_column(&col);
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

pub fn update_category_name(new_name: String, id: String) {
    println!("Update Category {:?}", id);
    let mut _id = "".to_string();
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!(
        "SELECT name, cast(id as text) id FROM categories ORDER BY sort, name LIMIT {},1",
        id
    );
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        _id = row.read::<&str, _>(1).to_string();
    }

    let re = Regex::new(r"[^a-zA-Z\d\s:_-]").unwrap();
    let _clean_name = re.replace_all(&new_name, "");
    let sql2 = format!(
        "UPDATE categories SET name = '{0}' WHERE id = {1} ",
        new_name, _id
    );
    let r = connection.execute(&sql2).unwrap();
    //println!("{:?}",r);
    return r;
}

pub fn update_subcategory_name(new_name: String, id: String, cat_id: &str) {
    //println!("Update Sucategory");
    let mut _id = "".to_string();
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!("SELECT name ,cast(id as text) id FROM subcategories WHERE category_id = {0} ORDER BY sort, name  LIMIT {1},1", cat_id,id  );
    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        _id = row.read::<&str, _>(1).to_string();
    }

    let re = Regex::new(r"[^a-zA-Z\d\s:_-]").unwrap();
    let clean_name = re.replace_all(&new_name, "");
    let sql2 = format!(
        "UPDATE subcategories SET name = '{0}' WHERE id = {1}",
        clean_name, _id
    );
    println!("{:?}", sql2);
    let _r = connection.execute(&sql2).unwrap();
    return _r;
}

pub fn add_category() {
    println!("add cat row");
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!("INSERT INTO categories(name)VALUES('')");
    println!("{:?}", sql);
    let _r = connection.execute(&sql);
}

pub fn delete_category(id: &String) {
    println!("delete cat row");
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!("DELETE FROM categories WHERE id = {}", id);
    println!("{:?}", sql);
    let _r = connection.execute(&sql);

    let sql2 = format!("DELETE FROM subcategories WHERE category_id = {}", id);
    println!("{:?}", sql2);
    let _r = connection.execute(&sql2);
}

pub fn delete_subcategory(id: &String) -> String {
    println!("delete subcat row");
    let mut category_id = "".to_string();
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!(
        "SELECT cast(category_id as TEXT) catetgory_id FROM subcategories WHERE id = {0} LIMIT 1",
        id
    );
    //println!("{:?}", sql);

    let statement = connection.prepare(sql).unwrap();
    let mut cursor = statement.into_iter();
    while let Some(row) = cursor.next() {
        let row = row.unwrap();
        category_id = row.read::<&str, _>(0).to_string();
        let connection = sqlite::open(&database_path()).unwrap();
        let sql = format!("DELETE FROM subcategories WHERE id = {}", id);
        //println!("{:?}",sql);
        let _r = connection.execute(&sql);
        //println!("Category ID: {:?}", category_id );
    }

    return category_id;
}

pub fn add_subcategory(id: &String, textview: &String) {
    println!("add subcat row {:?}", id);
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!(
        "INSERT INTO subcategories(name,category_id,content)VALUES('',{0},'{1}')",
        id,
        textview.to_string()
    );
    println!("{:?}", sql);
    let _r = connection.execute(&sql);
}

pub fn update_subcategory(id: &String, textview: &String) {
    println!("add subcat row {:?}", id);
    let connection = sqlite::open(&database_path()).unwrap();
    let sql = format!(
        "UPDATE subcategories SET content = '{0}' WHERE id = {1}",
        textview.to_string(),
        id
    );
    println!("{:?}", sql);
    let _r = connection.execute(&sql);
}

pub fn get_textview(textview: &gtk::TextView) -> String {
    let buffer = textview.buffer().unwrap();
    let start = buffer.start_iter();
    let end = buffer.end_iter();
    let g = buffer.text(&start, &end, true).unwrap();
    let s = g.as_str().to_string();
    return s;
}
