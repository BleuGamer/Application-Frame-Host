use std::collections::HashSet;
use std::convert::TryFrom;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use bytes::Bytes;
use futures_util::future::{ok, Ready};
use regex::Regex;
use actix_web::{
    dev::{
        BodySize,
        MessageBody,
    
        ResponseBody,
    },
    error::{ Error, Result },
    http::{ HeaderName, StatusCode },
    // service::{ ServiceRequest, ServiceResponse };
    HttpResponse 
};
