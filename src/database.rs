use gtk::prelude::*;
use glib::clone;
use regex::Regex;
use gtk::{CellRendererText,TreeViewColumn,TreeView,ListStore};
use base64::{encode,decode};
use cryptostream::{read, write};
use openssl::symm::{Cipher, Crypter, Mode};
use std::io::Read;
use std::io::Write;
use blake2::VarBlake2b;
use blake2::digest::{Update, VariableOutput};


/*
CREATE TABLE `databases` (
	`id`	INTEGER PRIMARY KEY AUTOINCREMENT,
	`database`	TEXT DEFAULT '',
	`password`	TEXT DEFAULT '',
	`save_password`	INTEGER DEFAULT 0,
	`default`	INTEGER DEFAULT 0
);
*/
