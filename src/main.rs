#![feature(plugin, custom_derive)]
#![plugin(rocket_codegen)]

extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod spinner;

fn main() {
    rocket::ignite().mount("/spinner", spinner::get_routes()).launch();
}
