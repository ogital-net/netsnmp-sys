#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

#![allow(unknown_lints)]
#![allow(unnecessary_transmutes)]
#![allow(clippy::unnecessary_transmutes)] // For Clippy

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::mem::MaybeUninit;

    use crate::{
        netsnmp_session, snmp_free_pdu, snmp_pdu_create, snmp_sess_init, SNMP_MSG_GET,
    };

    #[test]
    fn test_snmp_sess_init() {
        let _session: netsnmp_session = unsafe {
            let mut sess = MaybeUninit::<netsnmp_session>::uninit();
            snmp_sess_init(sess.as_mut_ptr());
            sess.assume_init()
        };
    }

    #[test]
    fn test_snmp_pdu_create() {
        unsafe {
            let pdu = snmp_pdu_create(SNMP_MSG_GET);
            assert!(!pdu.is_null());
            snmp_free_pdu(pdu);
        }
    }
}
