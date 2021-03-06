use std::{
    cell::RefCell,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use protocol::{
    types::{Address, Hash, ServiceContext, ServiceContextParams},
    Bytes,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::{new_riscv_service, with_dispatcher_service};
use crate::types::{DeployPayload, ExecPayload, InterpreterType};

const CYCLE_LIMIT: u64 = 1024 * 1024 * 1024;
const CALLER: &str = "0x0000000000000000000000000000000000000001";

struct TestContext {
    count:  usize,
    height: u64,
}

impl Default for TestContext {
    fn default() -> Self {
        TestContext {
            count:  1,
            height: 1,
        }
    }
}

impl TestContext {
    fn make(&mut self) -> ServiceContext {
        ServiceContext::new(self.new_params())
    }

    fn new_params(&mut self) -> ServiceContextParams {
        self.count += 1;
        self.height += 1;

        let tx_hash = Hash::digest(Bytes::from(format!("{}", self.count)));

        ServiceContextParams {
            tx_hash:         Some(tx_hash),
            nonce:           None,
            cycles_limit:    CYCLE_LIMIT,
            cycles_price:    1,
            cycles_used:     Rc::new(RefCell::new(3)),
            caller:          Address::from_hex(CALLER).expect("ctx caller"),
            height:          self.height,
            timestamp:       0,
            extra:           None,
            service_name:    "service_name".to_owned(),
            service_method:  "service_method".to_owned(),
            service_payload: "service_payload".to_owned(),
            events:          Rc::new(RefCell::new(vec![])),
        }
    }
}

macro_rules! deploy_test_code {
    () => {{
        let mut context = TestContext::default();
        let mut service = new_riscv_service();

        // No init
        let code = include_str!("./test_code.js");
        let payload = DeployPayload {
            code:      hex::encode(Bytes::from(code)),
            intp_type: InterpreterType::Duktape,
            init_args: "".into(),
        };

        let ret = service.deploy(context.make(), payload).expect("deploy");
        assert_eq!(ret.init_ret, "");

        (service, context, ret.address)
    }};
}

#[test]
fn should_support_pvm_init() {
    let (mut service, mut context, ..) = deploy_test_code!();

    let code = include_str!("./test_code.js");
    let payload = DeployPayload {
        code:      hex::encode(Bytes::from(code)),
        intp_type: InterpreterType::Duktape,
        init_args: "do init".into(),
    };

    let ret = service.deploy(context.make(), payload).expect("deploy");
    assert_eq!(ret.init_ret, "do init");
}

#[test]
fn should_support_pvm_load_args() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_load_args"}).to_string();
    let payload = ExecPayload::new(address, args.clone());

    let ret = service.exec(context.make(), payload).expect("load args");

    assert_eq!(ret, args);
}

#[test]
fn should_support_pvm_load_json_args() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_load_json_args"}).to_string();
    let payload = ExecPayload::new(address, args.clone());

    let ret = service
        .exec(context.make(), payload)
        .expect("load jsonn args");

    assert_eq!(ret, args);
}

#[test]
fn should_support_pvm_cycle_limit() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_limit"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service
        .exec(context.make(), payload)
        .expect("load cycle limit");

    assert_eq!(ret.parse::<u64>().expect("cycle limit"), CYCLE_LIMIT);
}

#[test]
fn should_support_pvm_cycle_used() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_used"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service.exec(ctx, payload).expect("load cycle used");

    // Hardcode in context make
    assert_eq!(ret.parse::<u64>().expect("cycle used"), 3);
}

#[test]
fn should_support_pvm_cycle_price() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_cycle_price"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service.exec(ctx, payload).expect("load cycle price");

    // Hardcode in context make
    assert_eq!(ret.parse::<u64>().expect("cycle price"), 1);
}

#[test]
fn should_support_pvm_caller() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_caller"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service.exec(context.make(), payload).expect("load caller");

    assert_eq!(format!("0x{}", ret), CALLER);
}

#[test]
fn should_support_pvm_origin() {
    let (mut service, mut context, address) = deploy_test_code!();

    // Deploy another test code
    let code = include_bytes!("./test_code.js");
    let payload = DeployPayload {
        code:      hex::encode(Bytes::from(code.as_ref())),
        intp_type: InterpreterType::Duktape,
        init_args: "".into(),
    };

    let tc_ctx = context.make();
    let tc_ret = with_dispatcher_service(move |dispatcher_service| {
        dispatcher_service.deploy(tc_ctx, payload)
    })
    .expect("deploy another test code");

    let args =
        json!({"method": "test_origin", "address": tc_ret.address.as_hex(), "call_args": json!({"method": "_ret_caller_and_origin"}).to_string()})
            .to_string();

    let payload = ExecPayload::new(address.clone(), args);

    let ret = service
        .exec(context.make(), payload)
        .expect("call contract _ret_caller_and_origin");

    #[derive(Debug, Deserialize)]
    struct ExpectRet {
        caller: String,
        origin: String,
    }

    let ret: ExpectRet = serde_json::from_str(&ret).expect("decode test origin ret");
    assert_eq!(ret.caller, address.as_hex());
    assert_eq!(format!("0x{}", ret.origin), CALLER);
}

#[test]
fn should_support_pvm_address() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_address"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ret = service.exec(context.make(), payload).expect("load address");

    assert_eq!(ret, address.as_hex());
}

#[test]
fn should_support_pvm_block_height() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_block_height"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service
        .exec(ctx.clone(), payload)
        .expect("load block height");

    assert_eq!(
        ret.parse::<u64>().expect("block height"),
        ctx.get_current_height()
    );
}

#[test]
fn should_support_pvm_extra() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_no_extra"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ret = service
        .exec(context.make(), payload)
        .expect("test no extra");

    assert_eq!(ret, "no extra");

    // Should return extra data
    let extra = "final mixed ??? no !!!";
    let mut ctx_params = context.new_params();
    ctx_params.extra = Some(Bytes::from(extra));
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_extra"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service.exec(ctx, payload).expect("test extra");

    assert_eq!(ret, extra);
}

#[test]
fn should_support_pvm_timestamp() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_timestamp"}).to_string();
    let payload = ExecPayload::new(address, args);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("timestamp")
        .as_secs();

    let mut ctx_params = context.new_params();
    ctx_params.timestamp = now;
    let ctx = ServiceContext::new(ctx_params);

    let ret = service.exec(ctx.clone(), payload).expect("load timestamp");
    assert_eq!(ret.parse::<u64>().expect("timestamp"), ctx.get_timestamp());
}

#[test]
fn should_support_pvm_emit_event() {
    let (mut service, mut context, address) = deploy_test_code!();

    let msg = "emit test event";
    let args = json!({"method": "test_emit_event", "msg": msg}).to_string();
    let payload = ExecPayload::new(address, args);

    let ctx = context.make();
    let ret = service.exec(ctx.clone(), payload).expect("emit event");
    assert_eq!(ret, "emit success");

    let events = ctx.get_events();
    assert!(events.iter().any(|ev| ev.data == msg));
}

#[test]
fn should_support_pvm_tx_hash() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_tx_hash"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ctx = context.make();
    let ret = service.exec(ctx.clone(), payload).expect("test tx hash");

    assert_eq!(
        Some(ret),
        ctx.get_tx_hash().map(|h| h.as_hex()),
        "should return tx hash"
    );

    // No tx hash
    let mut ctx_params = context.new_params();
    ctx_params.tx_hash = None;
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_no_tx_hash"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service.exec(ctx, payload).expect("test no tx hash");

    assert_eq!(ret, "no tx hash");
}

#[test]
fn should_support_pvm_tx_nonce() {
    let (mut service, mut context, address) = deploy_test_code!();

    let args = json!({"method": "test_no_tx_nonce"}).to_string();
    let payload = ExecPayload::new(address.clone(), args);

    let ctx = context.make();
    let ret = service.exec(ctx, payload).expect("tx no nonce");

    assert_eq!(ret, "no tx nonce");

    // Should return tx nonce
    let mut ctx_params = context.new_params();
    ctx_params.nonce = Some(Hash::digest(Bytes::from("test_nonce".to_owned())));
    let ctx = ServiceContext::new(ctx_params);

    let args = json!({"method": "test_tx_nonce"}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service.exec(ctx.clone(), payload).expect("test tx nonce");

    assert_eq!(Some(ret), ctx.get_nonce().map(|n| n.as_hex()));
}

#[test]
fn should_support_pvm_storage() {
    let (mut service, mut context, address) = deploy_test_code!();

    #[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
    struct Carmen {
        color: String,
    }

    let carmen = json!({"color": "red"}).to_string();
    let args = json!({"method": "test_storage", "key": "carmen", "val": carmen}).to_string();
    let payload = ExecPayload::new(address, args);

    let ret = service.exec(context.make(), payload).expect("load storage");

    let ret: Carmen = serde_json::from_str(&ret).expect("get json storage");

    assert_eq!(ret.color, "red");
}

#[test]
fn should_support_pvm_contract_call() {
    let (mut service, mut context, address) = deploy_test_code!();

    // Deploy another test code
    let code = include_bytes!("./test_code.js");
    let payload = DeployPayload {
        code:      hex::encode(Bytes::from(code.as_ref())),
        intp_type: InterpreterType::Duktape,
        init_args: "".into(),
    };

    let tc_ctx = context.make();
    let tc_ret = with_dispatcher_service(move |dispatcher_service| {
        dispatcher_service.deploy(tc_ctx, payload)
    })
    .expect("deploy another test code");

    let args =
        json!({"method": "test_contract_call", "address": tc_ret.address.as_hex(), "call_args": json!({"method": "_ret_self"}).to_string()})
            .to_string();

    let payload = ExecPayload::new(address, args);

    let ret = service
        .exec(context.make(), payload)
        .expect("exec contract call");

    assert_eq!(ret, "self");
}

#[test]
fn should_support_pvm_service_call() {
    let (mut service, mut context, address) = deploy_test_code!();

    // Deploy another test code
    let code = include_bytes!("./test_code.js");
    let payload = DeployPayload {
        code:      hex::encode(Bytes::from(code.as_ref())),
        intp_type: InterpreterType::Duktape,
        init_args: "".into(),
    };

    let tc_ctx = context.make();
    let tc_ret = with_dispatcher_service(move |dispatcher_service| {
        dispatcher_service.deploy(tc_ctx, payload)
    })
    .expect("deploy another test code");

    let args = json!({
        "method": "test_service_call",
        "call_service": "riscv",
        "call_method": "exec",
        "call_payload": json!({
            "address": tc_ret.address.as_hex(),
            "args": json!({
                "method": "_ret_self",
            }).to_string(),
        }).to_string(),
    })
    .to_string();

    let payload = ExecPayload::new(address, args);

    let ret = service
        .exec(context.make(), payload)
        .expect("exec service call");

    assert_eq!(ret, "self");
}
