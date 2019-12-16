//! client API module
// std
use std::sync::{Arc, Mutex, RwLock};

// thread
use std::thread;

// crossbeam
use crossbeam::{unbounded, Receiver, Sender};

// virtual router
use crate::VirtualRouter;

// router
mod router;

// sessions
mod sessions;
use sessions::auth::auth_api_client;
use sessions::token::SessionToken;

// config
use crate::config;

// Constants
const NOT_FOUND: &str = "404 Not found";

/// Upstream API structure
pub struct UpstreamAPI {
    sender: Sender<FSMQueryResult>,     // channel for queries to fsm
    receiver: Receiver<FSMQueryResult>, // channel for fsm's queries results
}

/// Upstream API structure implementation
impl<'a> UpstreamAPI {
    // new() method
    pub fn new() -> UpstreamAPI {
        let (sender, receiver) = unbounded();
        UpstreamAPI { sender, receiver }
    }
    // spawn_thread method
    pub fn spawn_thread(
        &self,
        down_api: &DownstreamAPI,
        cfg: config::CConfig,
        vrs: &Vec<Arc<RwLock<VirtualRouter>>>,
    ) {
        // upstream transmit and receives channels
        let (utx, urx) = self.channels();

        // clone receives and transmit channels for queries/responses
        let qrx = down_api.q_receiver.clone();
        let rtx = down_api.r_sender.clone();

        // duplicate virtual routers vector
        let mut vrouters: Vec<Arc<RwLock<VirtualRouter>>> = Vec::new();
        for vr in vrs {
            vrouters.push(vr.clone());
        }

        // spawn Client API thread
        thread::spawn(|| capi_thread_loop(utx, urx, qrx, rtx, cfg, vrouters));
    }
    // channels() method
    // channels to virtual routers finite-state-machines
    pub fn channels(&self) -> (Sender<FSMQueryResult>, Receiver<FSMQueryResult>) {
        (self.sender.clone(), self.receiver.clone())
    }
}

/// FSMQueryResult structure
#[derive(Debug)]
pub struct FSMQueryResult {
    pub QRflag: bool,          // Query/Response flag
    pub group: u8,             // recipient group
    pub interface: String,     // recipient interface
    pub query: FSMQuery,       // recipient query
    pub response: FSMResponse, // recipient response
}

/// FSMQuery enumerator
#[derive(Debug)]
pub enum FSMQuery {
    GlobalAttr,
    States,
    Group,
    Priority,
    Interface,
    None,
}

/// FSMResponse enumerator
#[derive(Debug)]
pub enum FSMResponse {
    States(u8, String),
    Group(u8, String),
    Priority(u8, String),
    Interface(u8, String),
    Empty,
}

/// Downstream API structure
#[derive(Clone, StateData)]
pub struct DownstreamAPI {
    q_sender: Arc<Mutex<Sender<ClientAPIQuery>>>,
    q_receiver: Arc<Mutex<Receiver<ClientAPIQuery>>>,
    r_sender: Arc<Mutex<Sender<ClientAPIResponse>>>,
    r_receiver: Arc<Mutex<Receiver<ClientAPIResponse>>>,
}

/// Downstream API implementation
impl DownstreamAPI {
    // new() method
    pub fn new() -> Self {
        let (qtx, qrx) = unbounded();
        let (rtx, rrx) = unbounded();
        Self {
            q_sender: Arc::new(Mutex::new(qtx)),
            q_receiver: Arc::new(Mutex::new(qrx)),
            r_sender: Arc::new(Mutex::new(rtx)),
            r_receiver: Arc::new(Mutex::new(rrx)),
        }
    }
    // query() method
    pub fn query(&self, q: ClientAPIQuery) {
        let qtx = self.q_sender.lock().unwrap();
        // panic if send fails
        qtx.send(q).unwrap();
    }
    // answer() method
    pub fn read(&self) -> ClientAPIResponse {
        let rrx = self.r_receiver.lock().unwrap();
        let answer = rrx.recv().unwrap();
        answer
    }
}

/// ClientAPIQuery enumerator
pub enum ClientAPIQuery {
    AuthRequest(String, String),
    CfgGlobalAll(SessionToken),
    CfgVrrpAll(SessionToken),
    CfgProtoAll(SessionToken),
    RunGlobalAll(SessionToken),
    RunVRRPAll(SessionToken),
    RunVRRPGrp(SessionToken, u8),
    RunVRRPGrpIntf(SessionToken, u8, String),
    RunProtoAll(SessionToken),
    RunProtoStatic(SessionToken),
}

/// ClientAPIResponse enumerator
pub enum ClientAPIResponse {
    Unauthorized,
    AuthResponse(Option<SessionToken>),
    CfgGlobalAll(config::CConfig),
    CfgVrrpAll(Vec<config::VRConfig>),
    CfgProtoAll(config::Protocols),
    RunGlobalAll(ResponseGlobalAttr),
    RunVRRPAll(Vec<ResponseVRRPAttr>),
    RunVRRPGrp(Option<Vec<ResponseVRRPAttr>>),
    RunVRRPGrpIntf(Option<ResponseVRRPAttr>),
    RunProtoAll(Option<ResponseProtoAttr>),
    RunProtoStatic(Option<Vec<ResponseProtoStaticAttr>>),
}

/// ReponseGlobalAttr structure (Serialize-able)
#[derive(Serialize)]
pub struct ResponseGlobalAttr {
    debug: u8,
    timestamp: u8,
    timezone: u8,
    pid: String,
    working_dir: String,
    main_log: String,
    error_log: String,
}

/// ResponseVRRPAttr structure (Serialize-able)
#[derive(Serialize)]
pub struct ResponseVRRPAttr {
    virtual_ip: String,
    group: u8,
    interface: String,
    priority: u8,
    preempt: bool,
    state: String,
}

/// RunProtoAttr structure (Serialize-able)
#[derive(Serialize)]
pub struct ResponseProtoAttr {
    r#static: Option<Vec<ResponseProtoStaticAttr>>,
}

/// RunProtoStaticAttr structure (Serialize-able)
#[derive(Serialize)]
pub struct ResponseProtoStaticAttr {
    destination: String,
    mask: String,
    next_hop: String,
    metric: i16,
    mtu: u64,
}

// capi_thread_loop() function
pub fn capi_thread_loop(
    utx: Sender<FSMQueryResult>,
    urx: Receiver<FSMQueryResult>,
    qrx: Arc<Mutex<Receiver<ClientAPIQuery>>>,
    rtx: Arc<Mutex<Sender<ClientAPIResponse>>>,
    cfg: config::CConfig,
    vrs: Vec<Arc<RwLock<VirtualRouter>>>,
) {
    eprintln!("Upstream Client API thread now runs!");

    loop {
        // declare empty response
        let resp;

        // acquire mutex lock
        let qrx = qrx.lock().unwrap();

        // listen for downstream queries (blocking)
        let q = qrx.recv().unwrap();
        match q {
            ClientAPIQuery::AuthRequest(user, passwd) => {
                let r = auth_api_client(&cfg, user, passwd);
                resp = ClientAPIResponse::AuthResponse(r);
            }
            ClientAPIQuery::CfgGlobalAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_cfg_global_all(&cfg);
                    resp = ClientAPIResponse::CfgGlobalAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::CfgVrrpAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_cfg_vrrp_all(&cfg);
                    resp = ClientAPIResponse::CfgVrrpAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::CfgProtoAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_cfg_proto_all(&cfg);
                    resp = ClientAPIResponse::CfgProtoAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunGlobalAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_global_all(&cfg);
                    resp = ClientAPIResponse::RunGlobalAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunVRRPAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_vrrp_all(&vrs);
                    resp = ClientAPIResponse::RunVRRPAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunVRRPGrp(sess, gid) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_vrrp_grp(&vrs, gid);
                    resp = ClientAPIResponse::RunVRRPGrp(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunVRRPGrpIntf(sess, gid, intf) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_vrrp_grp_intf(&vrs, gid, intf);
                    resp = ClientAPIResponse::RunVRRPGrpIntf(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunProtoAll(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_proto_all(&vrs);
                    resp = ClientAPIResponse::RunProtoAll(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
            ClientAPIQuery::RunProtoStatic(sess) => match sess.validate() {
                Some(_) => {
                    let r = capi_req_run_proto_static(&vrs);
                    resp = ClientAPIResponse::RunProtoStatic(r);
                }
                None => {
                    resp = ClientAPIResponse::Unauthorized;
                }
            },
        }

        // send queries answer back
        let rtx = rtx.lock().unwrap();
        let _r = rtx.send(resp);
    }
}

// start_capi_app() function
/// start Client API Application server
pub fn capi_start_app(down_api: DownstreamAPI) {
    // spawn the API application server in a new thread
    thread::spawn(move || router::start(down_api));
}

// capi_req_cfg_global_all() function
fn capi_req_cfg_global_all(cfg: &config::CConfig) -> config::CConfig {
    // return the entire global configuration (cloned)
    cfg.clone()
}

// capi_req_cfg_vrrp_all() function
fn capi_req_cfg_vrrp_all(cfg: &config::CConfig) -> Vec<config::VRConfig> {
    // return the configured virtual routers vector (cloned)
    cfg.vrouter.as_ref().unwrap().to_vec()
}

// capi_req_cfg_proto_all() function
fn capi_req_cfg_proto_all(cfg: &config::CConfig) -> config::Protocols {
    // return the configured protocols (cloned)
    cfg.protocols.as_ref().unwrap().clone()
}

// capi_req_run_global_all() function
fn capi_req_run_global_all(cfg: &config::CConfig) -> ResponseGlobalAttr {
    // build response for effective global configuration
    let attrs = ResponseGlobalAttr {
        debug: cfg.debug(),
        timestamp: cfg.time_format(),
        timezone: cfg.time_zone(),
        pid: cfg.pid(),
        working_dir: cfg.working_dir(),
        main_log: cfg.main_log(),
        error_log: cfg.error_log(),
    };

    attrs
}

// capi_req_run_vrrp_all() function
fn capi_req_run_vrrp_all(vrs: &Vec<Arc<RwLock<VirtualRouter>>>) -> Vec<ResponseVRRPAttr> {
    // initialize a vector of VRRP response
    let mut vattrs: Vec<ResponseVRRPAttr> = Vec::new();
    // iterate through all virtual routers
    for vr in vrs {
        // get read access
        let vro = vr.read().unwrap();
        // build VRRP attributes response
        let attrs = ResponseVRRPAttr {
            virtual_ip: vro.parameters.attr_vip(),
            group: vro.parameters.vrid(),
            interface: vro.parameters.interface(),
            priority: vro.parameters.prio(),
            preempt: vro.parameters.preempt(),
            state: vro.states.states(),
        };
        // push attriburs into vector
        vattrs.push(attrs);
    }

    vattrs
}

// capi_req_run_vrrp_grp() function
fn capi_req_run_vrrp_grp(
    vrs: &Vec<Arc<RwLock<VirtualRouter>>>,
    gid: u8,
) -> Option<Vec<ResponseVRRPAttr>> {
    // initialize attributes vector
    let mut vaddrs: Vec<ResponseVRRPAttr> = Vec::new();

    // create a new iterator fvr with matched elements
    let fvr = vrs.iter().filter(|&vr| {
        let vr = vr.read().unwrap();
        vr.parameters.vrid() == gid
    });

    // build attributes vector
    for vr in fvr {
        // get read access
        let vr = vr.read().unwrap();
        // build VRRP attributes response
        let attrs = ResponseVRRPAttr {
            virtual_ip: vr.parameters.attr_vip(),
            group: vr.parameters.vrid(),
            interface: vr.parameters.interface(),
            priority: vr.parameters.prio(),
            preempt: vr.parameters.preempt(),
            state: vr.states.states(),
        };
        // push vr
        vaddrs.push(attrs);
    }

    Some(vaddrs)
}

// capi_req_run_vrrp_grp_intf() function
fn capi_req_run_vrrp_grp_intf(
    vrs: &Vec<Arc<RwLock<VirtualRouter>>>,
    gid: u8,
    intf: String,
) -> Option<ResponseVRRPAttr> {
    // find a virtual router matching the vrid (gid) and interface (intf)
    let r = vrs.iter().find(|&vr| {
        let vr = vr.read().unwrap();
        (vr.parameters.vrid() == gid) && (vr.parameters.interface() == intf)
    });

    // check if the result is a found vr
    match r {
        Some(vr) => {
            // get read access
            let vr = vr.read().unwrap();
            // build VRRP attributes response
            let attrs = ResponseVRRPAttr {
                virtual_ip: vr.parameters.attr_vip(),
                group: vr.parameters.vrid(),
                interface: vr.parameters.interface(),
                priority: vr.parameters.prio(),
                preempt: vr.parameters.preempt(),
                state: vr.states.states(),
            };
            // return vr's attributes
            Some(attrs)
        }
        // if there is no matching vr, return None
        None => Option::None,
    }
}

// capi_req_run_proto_all() function
fn capi_req_run_proto_all(vrs: &Vec<Arc<RwLock<VirtualRouter>>>) -> Option<ResponseProtoAttr> {
    // get static attributes vector (if any)
    match capi_req_run_proto_static(vrs) {
        Some(v) => {
            // build response of protocols attributes
            let attrs = ResponseProtoAttr { r#static: Some(v) };
            Some(attrs)
        }
        None => None,
    }
}

// capi_req_run_proto_static() function
fn capi_req_run_proto_static(
    vrs: &Vec<Arc<RwLock<VirtualRouter>>>,
) -> Option<Vec<ResponseProtoStaticAttr>> {
    // create static attributes vector
    let mut pattrs: Vec<ResponseProtoStaticAttr> = Vec::new();
    // access only first virtual router
    let vr = &vrs[0];
    // get read access
    let vro = vr.read().unwrap();
    // get access to protocols structure
    let protocols = vro.parameters.protocols();
    let vrp = protocols.lock().unwrap();
    // if static option has some element (of type vector)
    match &vrp.r#static {
        Some(stv) => {
            for st in stv {
                // build protocols attributes response
                let attrs = ResponseProtoStaticAttr {
                    destination: format!(
                        "{}.{}.{}.{}",
                        st.route()[0],
                        st.route()[1],
                        st.route()[2],
                        st.route()[3]
                    ),
                    mask: format!(
                        "{}.{}.{}.{}",
                        st.mask()[0],
                        st.mask()[1],
                        st.mask()[2],
                        st.mask()[3]
                    ),
                    next_hop: format!(
                        "{}.{}.{}.{}",
                        st.nh()[0],
                        st.nh()[1],
                        st.nh()[2],
                        st.nh()[3]
                    ),
                    metric: st.metric(),
                    mtu: st.mtu(),
                };
                // push static attributes in vector
                pattrs.push(attrs);
            }
            // return the vector
            return Some(pattrs);
        }
        None => return None,
    }
}
