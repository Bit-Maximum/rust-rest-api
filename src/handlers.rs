use iron::*;
use iron::mime::{Mime, TopLevel, SubLevel};
use postgres::Client;
use serde_json;

use std::io::Read;
use std::sync::Mutex;
use crate::db;
use crate::db::Record;

// Get all records that`s "name" argument match given template
pub fn get_records(sdb: &Mutex<Client>, request: &mut Request) -> IronResult<Response> {
    let url: url::Url = request.url.clone().into();
    let mut name: Option<String> = None;
    let qp = url.query_pairs();
    if qp.count() != 1 {
        return Ok(Response::with((status::BadRequest,
                                  "passed more than one parameter or no parameters at all")));
    }
    let (key, value) = qp.last().unwrap();
    if key == "name" {
        name = Some(value.to_string());
    }

    let json_records;
    if let Ok(records) = db::read(sdb, name.as_ref().map(|s| &s[..])) {
        if let Ok(json) = serde_json::to_string(&records) {
            json_records = Some(json);
        } else {
            return Ok(Response::with((status::InternalServerError,
                                      "couldn't convert records to JSON")));
        }
    } else {
        return Ok(Response::with((status::InternalServerError,
                                  "couldn't read records from database")));
    }
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());

    Ok(Response::with((content_type, status::Ok, json_records.unwrap())))
}


// Get record with ID
pub fn get_record(sdb: &Mutex<Client>, request: &mut Request) -> IronResult<Response> {
    let url: url::Url = request.url.clone().into();
    let path = url.path_segments().unwrap();
    let sid: &str = &path.last().unwrap();
    let id;
    if let Ok(r) = sid.parse() {
        id = r;
    } else {
        return Ok(Response::with((status::BadRequest, "bad id")));
    }

    let json_record;
    if let Ok(recs) = db::read_one(sdb, id) {
        if let Ok(json) = serde_json::to_string(&recs) {
            json_record = Some(json);
        } else {
            return Ok(Response::with((status::InternalServerError,
                                      "couldn't convert records to JSON")));
        }
    } else {
        return Ok(Response::with((status::InternalServerError,
                                  "couldn't read records from database")));
    }
    let content_type = Mime(TopLevel::Application, SubLevel::Json, Vec::new());

    Ok(Response::with((content_type, status::Ok, json_record.unwrap())))
}


// Add new record from given JSON parameters
pub fn add_record(sdb: &Mutex<Client>, request: &mut Request) -> IronResult<Response> {
    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let decoded: serde_json::Result<Record> = serde_json::from_str(&body);
    if let Ok(record) = decoded {
        if record.name == "" || record.phone == "" {
            return Ok(Response::with((status::BadRequest, "empty name or phone")));
        }
        if let Ok(_) = db::insert(&mut *sdb.lock().unwrap(), &record.name, &record.phone) {
            Ok(Response::with(status::Created))
        } else {
            Ok(Response::with((status::InternalServerError, "couldn't insert record")))
        }
    } else {
        return Ok(Response::with((status::BadRequest, "couldn't decode JSON")))
    }
}


// Update record with give ID. Put data give in request:body <JSON>
pub fn update_record(sdb: &Mutex<Client>, request: &mut Request) -> IronResult<Response> {
    let url: url::Url = request.url.clone().into();
    let path = url.path_segments().unwrap();
    let sid: &str = &path.last().unwrap();
    let id;
    if let Ok(r) = sid.parse() {
        id = r;
    } else {
        return Ok(Response::with((status::BadRequest, "bad id")));
    }

    let mut body = String::new();
    request.body.read_to_string(&mut body).unwrap();
    let decoded: serde_json::Result<Record> = serde_json::from_str(&body);
    if let Ok(record) = decoded {
        if record.name == "" || record.phone == "" {
            return Ok(Response::with((status::BadRequest, "empty name or phone")));
        }
        if let Ok(_) = db::update(&mut *sdb.lock().unwrap(), id, &record.name, &record.phone) {
            Ok(Response::with(status::NoContent))
        } else {
            Ok(Response::with((status::NotFound, "couldn't update record")))
        }
    } else {
        return Ok(Response::with((status::BadRequest, "couldn't decode JSON")));
    }
}


// Delete record with given ID
pub fn delete_record(sdb: &Mutex<Client>, request: &mut Request) -> IronResult<Response> {
    let url: url::Url = request.url.clone().into();
    let path = url.path_segments().unwrap();
    let sid: &str = &path.last().unwrap();
    let id;
    if let Ok(r) = sid.parse() {
        id = r;
    } else {
        return Ok(Response::with((status::BadRequest, "bad id")));
    }

    if let Ok(_) = db::remove(&mut *sdb.lock().unwrap(), &[id]) {
        Ok(Response::with(status::NoContent))
    } else {
        Ok(Response::with((status::NotFound, "couldn't delete record")))
    }
}