use std::ffi::{c_void, CString};
use windows::core::{PCSTR, PWSTR};
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::Wmi::*; // WBEM*_E_* consts
use windows::Win32::System::{
    Diagnostics::Debug::{
        FormatMessageW, FORMAT_MESSAGE_FROM_HMODULE, FORMAT_MESSAGE_FROM_SYSTEM,
        FORMAT_MESSAGE_IGNORE_INSERTS,
    },
    LibraryLoader::{LoadLibraryExA, LOAD_LIBRARY_SEARCH_SYSTEM32},
};

// https://learn.microsoft.com/en-us/windows/win32/debug/system-error-codes--0-499-
const ERROR_INSUFFICIENT_BUFFER: u32 = 0x7A;

/// Obtain the (potentially localised) message, if possible.
pub fn to_message(hres: i32) -> String {
    let module = CString::new("wbem\\wmiutils.dll").unwrap();
    let module = unsafe {
        LoadLibraryExA(
            PCSTR::from_raw(module.as_ptr() as *const u8),
            None,
            LOAD_LIBRARY_SEARCH_SYSTEM32,
        )
    };
    let module = match module {
        Ok(hnd) => Some(hnd.0 as *const c_void),
        Err(_) => None,
    };

    let flags = FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
    let mut fixed_buff = [0_u16; 512];
    let mut size = unsafe {
        // Try messages in wmiutils.dll
        FormatMessageW(
            flags | FORMAT_MESSAGE_FROM_HMODULE,
            module,
            hres as u32,
            0,
            PWSTR::from_raw(fixed_buff.as_mut_ptr()),
            fixed_buff.len() as u32 - 1,
            None,
        )
    };
    if size == 0 {
        let winerr = unsafe { GetLastError() };
        if winerr.0 != ERROR_INSUFFICIENT_BUFFER {
            // Try system messages
            size = unsafe {
                FormatMessageW(
                    flags,
                    None,
                    winerr.0,
                    0,
                    PWSTR::from_raw(fixed_buff.as_mut_ptr()),
                    fixed_buff.len() as u32 - 1,
                    None,
                )
            };
        }
    }

    if size > 0 && (size as usize) < fixed_buff.len() {
        String::from_utf16_lossy(&fixed_buff[0..size as usize])
    } else {
        String::new() // Message not found or buffer too small
    }
}

/// Return a hard-coded stringified constant or a useful categorisation.
pub const fn to_class(hres: i32) -> &'static str {
    match WBEMSTATUS(hres) {
        WBEM_E_FAILED => "WBEM_E_FAILED",
        WBEM_E_NOT_FOUND => "WBEM_E_NOT_FOUND",
        WBEM_E_ACCESS_DENIED => "WBEM_E_ACCESS_DENIED",
        WBEM_E_PROVIDER_FAILURE => "WBEM_E_PROVIDER_FAILURE",
        WBEM_E_TYPE_MISMATCH => "WBEM_E_TYPE_MISMATCH",
        WBEM_E_OUT_OF_MEMORY => "WBEM_E_OUT_OF_MEMORY",
        WBEM_E_INVALID_CONTEXT => "WBEM_E_INVALID_CONTEXT",
        WBEM_E_INVALID_PARAMETER => "WBEM_E_INVALID_PARAMETER",
        WBEM_E_NOT_AVAILABLE => "WBEM_E_NOT_AVAILABLE",
        WBEM_E_CRITICAL_ERROR => "WBEM_E_CRITICAL_ERROR",
        WBEM_E_INVALID_STREAM => "WBEM_E_INVALID_STREAM",
        WBEM_E_NOT_SUPPORTED => "WBEM_E_NOT_SUPPORTED",
        WBEM_E_INVALID_SUPERCLASS => "WBEM_E_INVALID_SUPERCLASS",
        WBEM_E_INVALID_NAMESPACE => "WBEM_E_INVALID_NAMESPACE",
        WBEM_E_INVALID_OBJECT => "WBEM_E_INVALID_OBJECT",
        WBEM_E_INVALID_CLASS => "WBEM_E_INVALID_CLASS",
        WBEM_E_PROVIDER_NOT_FOUND => "WBEM_E_PROVIDER_NOT_FOUND",
        WBEM_E_INVALID_PROVIDER_REGISTRATION => "WBEM_E_INVALID_PROVIDER_REGISTRATION",
        WBEM_E_PROVIDER_LOAD_FAILURE => "WBEM_E_PROVIDER_LOAD_FAILURE",
        WBEM_E_INITIALIZATION_FAILURE => "WBEM_E_INITIALIZATION_FAILURE",
        WBEM_E_TRANSPORT_FAILURE => "WBEM_E_TRANSPORT_FAILURE",
        WBEM_E_INVALID_OPERATION => "WBEM_E_INVALID_OPERATION",
        WBEM_E_INVALID_QUERY => "WBEM_E_INVALID_QUERY",
        WBEM_E_INVALID_QUERY_TYPE => "WBEM_E_INVALID_QUERY_TYPE",
        WBEM_E_ALREADY_EXISTS => "WBEM_E_ALREADY_EXISTS",
        WBEM_E_OVERRIDE_NOT_ALLOWED => "WBEM_E_OVERRIDE_NOT_ALLOWED",
        WBEM_E_PROPAGATED_QUALIFIER => "WBEM_E_PROPAGATED_QUALIFIER",
        WBEM_E_PROPAGATED_PROPERTY => "WBEM_E_PROPAGATED_PROPERTY",
        WBEM_E_UNEXPECTED => "WBEM_E_UNEXPECTED",
        WBEM_E_ILLEGAL_OPERATION => "WBEM_E_ILLEGAL_OPERATION",
        WBEM_E_CANNOT_BE_KEY => "WBEM_E_CANNOT_BE_KEY",
        WBEM_E_INCOMPLETE_CLASS => "WBEM_E_INCOMPLETE_CLASS",
        WBEM_E_INVALID_SYNTAX => "WBEM_E_INVALID_SYNTAX",
        WBEM_E_NONDECORATED_OBJECT => "WBEM_E_NONDECORATED_OBJECT",
        WBEM_E_READ_ONLY => "WBEM_E_READ_ONLY",
        WBEM_E_PROVIDER_NOT_CAPABLE => "WBEM_E_PROVIDER_NOT_CAPABLE",
        WBEM_E_CLASS_HAS_CHILDREN => "WBEM_E_CLASS_HAS_CHILDREN",
        WBEM_E_CLASS_HAS_INSTANCES => "WBEM_E_CLASS_HAS_INSTANCES",
        WBEM_E_QUERY_NOT_IMPLEMENTED => "WBEM_E_QUERY_NOT_IMPLEMENTED",
        WBEM_E_ILLEGAL_NULL => "WBEM_E_ILLEGAL_NULL",
        WBEM_E_INVALID_QUALIFIER_TYPE => "WBEM_E_INVALID_QUALIFIER_TYPE",
        WBEM_E_INVALID_PROPERTY_TYPE => "WBEM_E_INVALID_PROPERTY_TYPE",
        WBEM_E_VALUE_OUT_OF_RANGE => "WBEM_E_VALUE_OUT_OF_RANGE",
        WBEM_E_CANNOT_BE_SINGLETON => "WBEM_E_CANNOT_BE_SINGLETON",
        WBEM_E_INVALID_CIM_TYPE => "WBEM_E_INVALID_CIM_TYPE",
        WBEM_E_INVALID_METHOD => "WBEM_E_INVALID_METHOD",
        WBEM_E_INVALID_METHOD_PARAMETERS => "WBEM_E_INVALID_METHOD_PARAMETERS",
        WBEM_E_SYSTEM_PROPERTY => "WBEM_E_SYSTEM_PROPERTY",
        WBEM_E_INVALID_PROPERTY => "WBEM_E_INVALID_PROPERTY",
        WBEM_E_CALL_CANCELLED => "WBEM_E_CALL_CANCELLED",
        WBEM_E_SHUTTING_DOWN => "WBEM_E_SHUTTING_DOWN",
        WBEM_E_PROPAGATED_METHOD => "WBEM_E_PROPAGATED_METHOD",
        WBEM_E_UNSUPPORTED_PARAMETER => "WBEM_E_UNSUPPORTED_PARAMETER",
        WBEM_E_MISSING_PARAMETER_ID => "WBEM_E_MISSING_PARAMETER_ID",
        WBEM_E_INVALID_PARAMETER_ID => "WBEM_E_INVALID_PARAMETER_ID",
        WBEM_E_NONCONSECUTIVE_PARAMETER_IDS => "WBEM_E_NONCONSECUTIVE_PARAMETER_IDS",
        WBEM_E_PARAMETER_ID_ON_RETVAL => "WBEM_E_PARAMETER_ID_ON_RETVAL",
        WBEM_E_INVALID_OBJECT_PATH => "WBEM_E_INVALID_OBJECT_PATH",
        WBEM_E_OUT_OF_DISK_SPACE => "WBEM_E_OUT_OF_DISK_SPACE",
        WBEM_E_BUFFER_TOO_SMALL => "WBEM_E_BUFFER_TOO_SMALL",
        WBEM_E_UNSUPPORTED_PUT_EXTENSION => "WBEM_E_UNSUPPORTED_PUT_EXTENSION",
        WBEM_E_UNKNOWN_OBJECT_TYPE => "WBEM_E_UNKNOWN_OBJECT_TYPE",
        WBEM_E_UNKNOWN_PACKET_TYPE => "WBEM_E_UNKNOWN_PACKET_TYPE",
        WBEM_E_MARSHAL_VERSION_MISMATCH => "WBEM_E_MARSHAL_VERSION_MISMATCH",
        WBEM_E_MARSHAL_INVALID_SIGNATURE => "WBEM_E_MARSHAL_INVALID_SIGNATURE",
        WBEM_E_INVALID_QUALIFIER => "WBEM_E_INVALID_QUALIFIER",
        WBEM_E_INVALID_DUPLICATE_PARAMETER => "WBEM_E_INVALID_DUPLICATE_PARAMETER",
        WBEM_E_TOO_MUCH_DATA => "WBEM_E_TOO_MUCH_DATA",
        WBEM_E_SERVER_TOO_BUSY => "WBEM_E_SERVER_TOO_BUSY",
        WBEM_E_INVALID_FLAVOR => "WBEM_E_INVALID_FLAVOR",
        WBEM_E_CIRCULAR_REFERENCE => "WBEM_E_CIRCULAR_REFERENCE",
        WBEM_E_UNSUPPORTED_CLASS_UPDATE => "WBEM_E_UNSUPPORTED_CLASS_UPDATE",
        WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE => "WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE",
        WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE => "WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE",
        WBEM_E_TOO_MANY_PROPERTIES => "WBEM_E_TOO_MANY_PROPERTIES",
        WBEM_E_UPDATE_TYPE_MISMATCH => "WBEM_E_UPDATE_TYPE_MISMATCH",
        WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED => "WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED",
        WBEM_E_UPDATE_PROPAGATED_METHOD => "WBEM_E_UPDATE_PROPAGATED_METHOD",
        WBEM_E_METHOD_NOT_IMPLEMENTED => "WBEM_E_METHOD_NOT_IMPLEMENTED",
        WBEM_E_METHOD_DISABLED => "WBEM_E_METHOD_DISABLED",
        WBEM_E_REFRESHER_BUSY => "WBEM_E_REFRESHER_BUSY",
        WBEM_E_UNPARSABLE_QUERY => "WBEM_E_UNPARSABLE_QUERY",
        WBEM_E_NOT_EVENT_CLASS => "WBEM_E_NOT_EVENT_CLASS",
        WBEM_E_MISSING_GROUP_WITHIN => "WBEM_E_MISSING_GROUP_WITHIN",
        WBEM_E_MISSING_AGGREGATION_LIST => "WBEM_E_MISSING_AGGREGATION_LIST",
        WBEM_E_PROPERTY_NOT_AN_OBJECT => "WBEM_E_PROPERTY_NOT_AN_OBJECT",
        WBEM_E_AGGREGATING_BY_OBJECT => "WBEM_E_AGGREGATING_BY_OBJECT",
        WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY => "WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY",
        WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING => "WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING",
        WBEM_E_QUEUE_OVERFLOW => "WBEM_E_QUEUE_OVERFLOW",
        WBEM_E_PRIVILEGE_NOT_HELD => "WBEM_E_PRIVILEGE_NOT_HELD",
        WBEM_E_INVALID_OPERATOR => "WBEM_E_INVALID_OPERATOR",
        WBEM_E_LOCAL_CREDENTIALS => "WBEM_E_LOCAL_CREDENTIALS",
        WBEM_E_CANNOT_BE_ABSTRACT => "WBEM_E_CANNOT_BE_ABSTRACT",
        WBEM_E_AMENDED_OBJECT => "WBEM_E_AMENDED_OBJECT",
        WBEM_E_CLIENT_TOO_SLOW => "WBEM_E_CLIENT_TOO_SLOW",
        WBEM_E_NULL_SECURITY_DESCRIPTOR => "WBEM_E_NULL_SECURITY_DESCRIPTOR",
        WBEM_E_TIMED_OUT => "WBEM_E_TIMED_OUT",
        WBEM_E_INVALID_ASSOCIATION => "WBEM_E_INVALID_ASSOCIATION",
        WBEM_E_AMBIGUOUS_OPERATION => "WBEM_E_AMBIGUOUS_OPERATION",
        WBEM_E_QUOTA_VIOLATION => "WBEM_E_QUOTA_VIOLATION",
        WBEM_E_TRANSACTION_CONFLICT => "WBEM_E_TRANSACTION_CONFLICT",
        WBEM_E_FORCED_ROLLBACK => "WBEM_E_FORCED_ROLLBACK",
        WBEM_E_UNSUPPORTED_LOCALE => "WBEM_E_UNSUPPORTED_LOCALE",
        WBEM_E_HANDLE_OUT_OF_DATE => "WBEM_E_HANDLE_OUT_OF_DATE",
        WBEM_E_CONNECTION_FAILED => "WBEM_E_CONNECTION_FAILED",
        WBEM_E_INVALID_HANDLE_REQUEST => "WBEM_E_INVALID_HANDLE_REQUEST",
        WBEM_E_PROPERTY_NAME_TOO_WIDE => "WBEM_E_PROPERTY_NAME_TOO_WIDE",
        WBEM_E_CLASS_NAME_TOO_WIDE => "WBEM_E_CLASS_NAME_TOO_WIDE",
        WBEM_E_METHOD_NAME_TOO_WIDE => "WBEM_E_METHOD_NAME_TOO_WIDE",
        WBEM_E_QUALIFIER_NAME_TOO_WIDE => "WBEM_E_QUALIFIER_NAME_TOO_WIDE",
        WBEM_E_RERUN_COMMAND => "WBEM_E_RERUN_COMMAND",
        WBEM_E_DATABASE_VER_MISMATCH => "WBEM_E_DATABASE_VER_MISMATCH",
        WBEM_E_VETO_DELETE => "WBEM_E_VETO_DELETE",
        WBEM_E_VETO_PUT => "WBEM_E_VETO_PUT",
        WBEM_E_INVALID_LOCALE => "WBEM_E_INVALID_LOCALE",
        WBEM_E_PROVIDER_SUSPENDED => "WBEM_E_PROVIDER_SUSPENDED",
        WBEM_E_SYNCHRONIZATION_REQUIRED => "WBEM_E_SYNCHRONIZATION_REQUIRED",
        WBEM_E_NO_SCHEMA => "WBEM_E_NO_SCHEMA",
        WBEM_E_PROVIDER_ALREADY_REGISTERED => "WBEM_E_PROVIDER_ALREADY_REGISTERED",
        WBEM_E_PROVIDER_NOT_REGISTERED => "WBEM_E_PROVIDER_NOT_REGISTERED",
        WBEM_E_FATAL_TRANSPORT_ERROR => "WBEM_E_FATAL_TRANSPORT_ERROR",
        WBEM_E_ENCRYPTED_CONNECTION_REQUIRED => "WBEM_E_ENCRYPTED_CONNECTION_REQUIRED",
        WBEM_E_PROVIDER_TIMED_OUT => "WBEM_E_PROVIDER_TIMED_OUT",
        WBEM_E_NO_KEY => "WBEM_E_NO_KEY",
        WBEM_E_PROVIDER_DISABLED => "WBEM_E_PROVIDER_DISABLED",
        WBEMESS_E_REGISTRATION_TOO_BROAD => "WBEMESS_E_REGISTRATION_TOO_BROAD",
        WBEMESS_E_REGISTRATION_TOO_PRECISE => "WBEMESS_E_REGISTRATION_TOO_PRECISE",
        WBEMESS_E_AUTHZ_NOT_PRIVILEGED => "WBEMESS_E_AUTHZ_NOT_PRIVILEGED",
        WBEMMOF_E_EXPECTED_QUALIFIER_NAME => "WBEMMOF_E_EXPECTED_QUALIFIER_NAME",
        WBEMMOF_E_EXPECTED_SEMI => "WBEMMOF_E_EXPECTED_SEMI",
        WBEMMOF_E_EXPECTED_OPEN_BRACE => "WBEMMOF_E_EXPECTED_OPEN_BRACE",
        WBEMMOF_E_EXPECTED_CLOSE_BRACE => "WBEMMOF_E_EXPECTED_CLOSE_BRACE",
        WBEMMOF_E_EXPECTED_CLOSE_BRACKET => "WBEMMOF_E_EXPECTED_CLOSE_BRACKET",
        WBEMMOF_E_EXPECTED_CLOSE_PAREN => "WBEMMOF_E_EXPECTED_CLOSE_PAREN",
        WBEMMOF_E_ILLEGAL_CONSTANT_VALUE => "WBEMMOF_E_ILLEGAL_CONSTANT_VALUE",
        WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER => "WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER",
        WBEMMOF_E_EXPECTED_OPEN_PAREN => "WBEMMOF_E_EXPECTED_OPEN_PAREN",
        WBEMMOF_E_UNRECOGNIZED_TOKEN => "WBEMMOF_E_UNRECOGNIZED_TOKEN",
        WBEMMOF_E_UNRECOGNIZED_TYPE => "WBEMMOF_E_UNRECOGNIZED_TYPE",
        WBEMMOF_E_EXPECTED_PROPERTY_NAME => "WBEMMOF_E_EXPECTED_PROPERTY_NAME",
        WBEMMOF_E_TYPEDEF_NOT_SUPPORTED => "WBEMMOF_E_TYPEDEF_NOT_SUPPORTED",
        WBEMMOF_E_UNEXPECTED_ALIAS => "WBEMMOF_E_UNEXPECTED_ALIAS",
        WBEMMOF_E_UNEXPECTED_ARRAY_INIT => "WBEMMOF_E_UNEXPECTED_ARRAY_INIT",
        WBEMMOF_E_INVALID_AMENDMENT_SYNTAX => "WBEMMOF_E_INVALID_AMENDMENT_SYNTAX",
        WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT => "WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT",
        WBEMMOF_E_INVALID_PRAGMA => "WBEMMOF_E_INVALID_PRAGMA",
        WBEMMOF_E_INVALID_NAMESPACE_SYNTAX => "WBEMMOF_E_INVALID_NAMESPACE_SYNTAX",
        WBEMMOF_E_EXPECTED_CLASS_NAME => "WBEMMOF_E_EXPECTED_CLASS_NAME",
        WBEMMOF_E_TYPE_MISMATCH => "WBEMMOF_E_TYPE_MISMATCH",
        WBEMMOF_E_EXPECTED_ALIAS_NAME => "WBEMMOF_E_EXPECTED_ALIAS_NAME",
        WBEMMOF_E_INVALID_CLASS_DECLARATION => "WBEMMOF_E_INVALID_CLASS_DECLARATION",
        WBEMMOF_E_INVALID_INSTANCE_DECLARATION => "WBEMMOF_E_INVALID_INSTANCE_DECLARATION",
        WBEMMOF_E_EXPECTED_DOLLAR => "WBEMMOF_E_EXPECTED_DOLLAR",
        WBEMMOF_E_CIMTYPE_QUALIFIER => "WBEMMOF_E_CIMTYPE_QUALIFIER",
        WBEMMOF_E_DUPLICATE_PROPERTY => "WBEMMOF_E_DUPLICATE_PROPERTY",
        WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION => "WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION",
        WBEMMOF_E_OUT_OF_RANGE => "WBEMMOF_E_OUT_OF_RANGE",
        WBEMMOF_E_INVALID_FILE => "WBEMMOF_E_INVALID_FILE",
        WBEMMOF_E_ALIASES_IN_EMBEDDED => "WBEMMOF_E_ALIASES_IN_EMBEDDED",
        WBEMMOF_E_NULL_ARRAY_ELEM => "WBEMMOF_E_NULL_ARRAY_ELEM",
        WBEMMOF_E_DUPLICATE_QUALIFIER => "WBEMMOF_E_DUPLICATE_QUALIFIER",
        WBEMMOF_E_EXPECTED_FLAVOR_TYPE => "WBEMMOF_E_EXPECTED_FLAVOR_TYPE",
        WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES => "WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES",
        WBEMMOF_E_MULTIPLE_ALIASES => "WBEMMOF_E_MULTIPLE_ALIASES",
        WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2 => "WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2",
        WBEMMOF_E_NO_ARRAYS_RETURNED => "WBEMMOF_E_NO_ARRAYS_RETURNED",
        WBEMMOF_E_MUST_BE_IN_OR_OUT => "WBEMMOF_E_MUST_BE_IN_OR_OUT",
        WBEMMOF_E_INVALID_FLAGS_SYNTAX => "WBEMMOF_E_INVALID_FLAGS_SYNTAX",
        WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE => "WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE",
        WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE => "WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE",
        WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE => "WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE",
        WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX => "WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX",
        WBEMMOF_E_INVALID_QUALIFIER_SYNTAX => "WBEMMOF_E_INVALID_QUALIFIER_SYNTAX",
        WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE => "WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE",
        WBEMMOF_E_ERROR_CREATING_TEMP_FILE => "WBEMMOF_E_ERROR_CREATING_TEMP_FILE",
        WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE => "WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE",
        WBEMMOF_E_INVALID_DELETECLASS_SYNTAX => "WBEMMOF_E_INVALID_DELETECLASS_SYNTAX",
        _ => match WBEM_EXTRA_RETURN_CODES(hres) {
            WBEM_E_RETRY_LATER => "WBEM_E_RETRY_LATER",
            WBEM_E_RESOURCE_CONTENTION => "WBEM_E_RESOURCE_CONTENTION",
            _ => match hres as u32 {
                x if x >= 0x80041068 && x <= 0x80041099 => "WMI",
                x if x >= 0x80070000 && x <= 0x80079999 => "OS",
                x if x >= 0x80040000 && x <= 0x80040999 => "DCOM",
                x if x >= 0x80050000 && x <= 0x80059999 => "ADSI/LDAP",
                _ => "UNKNOWN",
            },
        },
    }
}

/// Return a hard-coded English description, if possible.
pub const fn to_detail(hres: i32) -> &'static str {
    match WBEMSTATUS(hres) {
        WBEM_E_FAILED => WBEM_E_FAILED_EN,
        WBEM_E_NOT_FOUND => WBEM_E_NOT_FOUND_EN,
        WBEM_E_ACCESS_DENIED => WBEM_E_ACCESS_DENIED_EN,
        WBEM_E_PROVIDER_FAILURE => WBEM_E_PROVIDER_FAILURE_EN,
        WBEM_E_TYPE_MISMATCH => WBEM_E_TYPE_MISMATCH_EN,
        WBEM_E_OUT_OF_MEMORY => WBEM_E_OUT_OF_MEMORY_EN,
        WBEM_E_INVALID_CONTEXT => WBEM_E_INVALID_CONTEXT_EN,
        WBEM_E_INVALID_PARAMETER => WBEM_E_INVALID_PARAMETER_EN,
        WBEM_E_NOT_AVAILABLE => WBEM_E_NOT_AVAILABLE_EN,
        WBEM_E_CRITICAL_ERROR => WBEM_E_CRITICAL_ERROR_EN,
        WBEM_E_INVALID_STREAM => WBEM_E_INVALID_STREAM_EN,
        WBEM_E_NOT_SUPPORTED => WBEM_E_NOT_SUPPORTED_EN,
        WBEM_E_INVALID_SUPERCLASS => WBEM_E_INVALID_SUPERCLASS_EN,
        WBEM_E_INVALID_NAMESPACE => WBEM_E_INVALID_NAMESPACE_EN,
        WBEM_E_INVALID_OBJECT => WBEM_E_INVALID_OBJECT_EN,
        WBEM_E_INVALID_CLASS => WBEM_E_INVALID_CLASS_EN,
        WBEM_E_PROVIDER_NOT_FOUND => WBEM_E_PROVIDER_NOT_FOUND_EN,
        WBEM_E_INVALID_PROVIDER_REGISTRATION => WBEM_E_INVALID_PROVIDER_REGISTRATION_EN,
        WBEM_E_PROVIDER_LOAD_FAILURE => WBEM_E_PROVIDER_LOAD_FAILURE_EN,
        WBEM_E_INITIALIZATION_FAILURE => WBEM_E_INITIALIZATION_FAILURE_EN,
        WBEM_E_TRANSPORT_FAILURE => WBEM_E_TRANSPORT_FAILURE_EN,
        WBEM_E_INVALID_OPERATION => WBEM_E_INVALID_OPERATION_EN,
        WBEM_E_INVALID_QUERY => WBEM_E_INVALID_QUERY_EN,
        WBEM_E_INVALID_QUERY_TYPE => WBEM_E_INVALID_QUERY_TYPE_EN,
        WBEM_E_ALREADY_EXISTS => WBEM_E_ALREADY_EXISTS_EN,
        WBEM_E_OVERRIDE_NOT_ALLOWED => WBEM_E_OVERRIDE_NOT_ALLOWED_EN,
        WBEM_E_PROPAGATED_QUALIFIER => WBEM_E_PROPAGATED_QUALIFIER_EN,
        WBEM_E_PROPAGATED_PROPERTY => WBEM_E_PROPAGATED_PROPERTY_EN,
        WBEM_E_UNEXPECTED => WBEM_E_UNEXPECTED_EN,
        WBEM_E_ILLEGAL_OPERATION => WBEM_E_ILLEGAL_OPERATION_EN,
        WBEM_E_CANNOT_BE_KEY => WBEM_E_CANNOT_BE_KEY_EN,
        WBEM_E_INCOMPLETE_CLASS => WBEM_E_INCOMPLETE_CLASS_EN,
        WBEM_E_INVALID_SYNTAX => WBEM_E_INVALID_SYNTAX_EN,
        WBEM_E_NONDECORATED_OBJECT => WBEM_E_NONDECORATED_OBJECT_EN,
        WBEM_E_READ_ONLY => WBEM_E_READ_ONLY_EN,
        WBEM_E_PROVIDER_NOT_CAPABLE => WBEM_E_PROVIDER_NOT_CAPABLE_EN,
        WBEM_E_CLASS_HAS_CHILDREN => WBEM_E_CLASS_HAS_CHILDREN_EN,
        WBEM_E_CLASS_HAS_INSTANCES => WBEM_E_CLASS_HAS_INSTANCES_EN,
        WBEM_E_QUERY_NOT_IMPLEMENTED => WBEM_E_QUERY_NOT_IMPLEMENTED_EN,
        WBEM_E_ILLEGAL_NULL => WBEM_E_ILLEGAL_NULL_EN,
        WBEM_E_INVALID_QUALIFIER_TYPE => WBEM_E_INVALID_QUALIFIER_TYPE_EN,
        WBEM_E_INVALID_PROPERTY_TYPE => WBEM_E_INVALID_PROPERTY_TYPE_EN,
        WBEM_E_VALUE_OUT_OF_RANGE => WBEM_E_VALUE_OUT_OF_RANGE_EN,
        WBEM_E_CANNOT_BE_SINGLETON => WBEM_E_CANNOT_BE_SINGLETON_EN,
        WBEM_E_INVALID_CIM_TYPE => WBEM_E_INVALID_CIM_TYPE_EN,
        WBEM_E_INVALID_METHOD => WBEM_E_INVALID_METHOD_EN,
        WBEM_E_INVALID_METHOD_PARAMETERS => WBEM_E_INVALID_METHOD_PARAMETERS_EN,
        WBEM_E_SYSTEM_PROPERTY => WBEM_E_SYSTEM_PROPERTY_EN,
        WBEM_E_INVALID_PROPERTY => WBEM_E_INVALID_PROPERTY_EN,
        WBEM_E_CALL_CANCELLED => WBEM_E_CALL_CANCELLED_EN,
        WBEM_E_SHUTTING_DOWN => WBEM_E_SHUTTING_DOWN_EN,
        WBEM_E_PROPAGATED_METHOD => WBEM_E_PROPAGATED_METHOD_EN,
        WBEM_E_UNSUPPORTED_PARAMETER => WBEM_E_UNSUPPORTED_PARAMETER_EN,
        WBEM_E_MISSING_PARAMETER_ID => WBEM_E_MISSING_PARAMETER_ID_EN,
        WBEM_E_INVALID_PARAMETER_ID => WBEM_E_INVALID_PARAMETER_ID_EN,
        WBEM_E_NONCONSECUTIVE_PARAMETER_IDS => WBEM_E_NONCONSECUTIVE_PARAMETER_IDS_EN,
        WBEM_E_PARAMETER_ID_ON_RETVAL => WBEM_E_PARAMETER_ID_ON_RETVAL_EN,
        WBEM_E_INVALID_OBJECT_PATH => WBEM_E_INVALID_OBJECT_PATH_EN,
        WBEM_E_OUT_OF_DISK_SPACE => WBEM_E_OUT_OF_DISK_SPACE_EN,
        WBEM_E_BUFFER_TOO_SMALL => WBEM_E_BUFFER_TOO_SMALL_EN,
        WBEM_E_UNSUPPORTED_PUT_EXTENSION => WBEM_E_UNSUPPORTED_PUT_EXTENSION_EN,
        WBEM_E_UNKNOWN_OBJECT_TYPE => WBEM_E_UNKNOWN_OBJECT_TYPE_EN,
        WBEM_E_UNKNOWN_PACKET_TYPE => WBEM_E_UNKNOWN_PACKET_TYPE_EN,
        WBEM_E_MARSHAL_VERSION_MISMATCH => WBEM_E_MARSHAL_VERSION_MISMATCH_EN,
        WBEM_E_MARSHAL_INVALID_SIGNATURE => WBEM_E_MARSHAL_INVALID_SIGNATURE_EN,
        WBEM_E_INVALID_QUALIFIER => WBEM_E_INVALID_QUALIFIER_EN,
        WBEM_E_INVALID_DUPLICATE_PARAMETER => WBEM_E_INVALID_DUPLICATE_PARAMETER_EN,
        WBEM_E_TOO_MUCH_DATA => WBEM_E_TOO_MUCH_DATA_EN,
        WBEM_E_SERVER_TOO_BUSY => WBEM_E_SERVER_TOO_BUSY_EN,
        WBEM_E_INVALID_FLAVOR => WBEM_E_INVALID_FLAVOR_EN,
        WBEM_E_CIRCULAR_REFERENCE => WBEM_E_CIRCULAR_REFERENCE_EN,
        WBEM_E_UNSUPPORTED_CLASS_UPDATE => WBEM_E_UNSUPPORTED_CLASS_UPDATE_EN,
        WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE => WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE_EN,
        WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE => WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE_EN,
        WBEM_E_TOO_MANY_PROPERTIES => WBEM_E_TOO_MANY_PROPERTIES_EN,
        WBEM_E_UPDATE_TYPE_MISMATCH => WBEM_E_UPDATE_TYPE_MISMATCH_EN,
        WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED => WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED_EN,
        WBEM_E_UPDATE_PROPAGATED_METHOD => WBEM_E_UPDATE_PROPAGATED_METHOD_EN,
        WBEM_E_METHOD_NOT_IMPLEMENTED => WBEM_E_METHOD_NOT_IMPLEMENTED_EN,
        WBEM_E_METHOD_DISABLED => WBEM_E_METHOD_DISABLED_EN,
        WBEM_E_REFRESHER_BUSY => WBEM_E_REFRESHER_BUSY_EN,
        WBEM_E_UNPARSABLE_QUERY => WBEM_E_UNPARSABLE_QUERY_EN,
        WBEM_E_NOT_EVENT_CLASS => WBEM_E_NOT_EVENT_CLASS_EN,
        WBEM_E_MISSING_GROUP_WITHIN => WBEM_E_MISSING_GROUP_WITHIN_EN,
        WBEM_E_MISSING_AGGREGATION_LIST => WBEM_E_MISSING_AGGREGATION_LIST_EN,
        WBEM_E_PROPERTY_NOT_AN_OBJECT => WBEM_E_PROPERTY_NOT_AN_OBJECT_EN,
        WBEM_E_AGGREGATING_BY_OBJECT => WBEM_E_AGGREGATING_BY_OBJECT_EN,
        WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY => WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY_EN,
        WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING => WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING_EN,
        WBEM_E_QUEUE_OVERFLOW => WBEM_E_QUEUE_OVERFLOW_EN,
        WBEM_E_PRIVILEGE_NOT_HELD => WBEM_E_PRIVILEGE_NOT_HELD_EN,
        WBEM_E_INVALID_OPERATOR => WBEM_E_INVALID_OPERATOR_EN,
        WBEM_E_LOCAL_CREDENTIALS => WBEM_E_LOCAL_CREDENTIALS_EN,
        WBEM_E_CANNOT_BE_ABSTRACT => WBEM_E_CANNOT_BE_ABSTRACT_EN,
        WBEM_E_AMENDED_OBJECT => WBEM_E_AMENDED_OBJECT_EN,
        WBEM_E_CLIENT_TOO_SLOW => WBEM_E_CLIENT_TOO_SLOW_EN,
        WBEM_E_NULL_SECURITY_DESCRIPTOR => WBEM_E_NULL_SECURITY_DESCRIPTOR_EN,
        WBEM_E_TIMED_OUT => WBEM_E_TIMED_OUT_EN,
        WBEM_E_INVALID_ASSOCIATION => WBEM_E_INVALID_ASSOCIATION_EN,
        WBEM_E_AMBIGUOUS_OPERATION => WBEM_E_AMBIGUOUS_OPERATION_EN,
        WBEM_E_QUOTA_VIOLATION => WBEM_E_QUOTA_VIOLATION_EN,
        WBEM_E_TRANSACTION_CONFLICT => WBEM_E_TRANSACTION_CONFLICT_EN,
        WBEM_E_FORCED_ROLLBACK => WBEM_E_FORCED_ROLLBACK_EN,
        WBEM_E_UNSUPPORTED_LOCALE => WBEM_E_UNSUPPORTED_LOCALE_EN,
        WBEM_E_HANDLE_OUT_OF_DATE => WBEM_E_HANDLE_OUT_OF_DATE_EN,
        WBEM_E_CONNECTION_FAILED => WBEM_E_CONNECTION_FAILED_EN,
        WBEM_E_INVALID_HANDLE_REQUEST => WBEM_E_INVALID_HANDLE_REQUEST_EN,
        WBEM_E_PROPERTY_NAME_TOO_WIDE => WBEM_E_PROPERTY_NAME_TOO_WIDE_EN,
        WBEM_E_CLASS_NAME_TOO_WIDE => WBEM_E_CLASS_NAME_TOO_WIDE_EN,
        WBEM_E_METHOD_NAME_TOO_WIDE => WBEM_E_METHOD_NAME_TOO_WIDE_EN,
        WBEM_E_QUALIFIER_NAME_TOO_WIDE => WBEM_E_QUALIFIER_NAME_TOO_WIDE_EN,
        WBEM_E_RERUN_COMMAND => WBEM_E_RERUN_COMMAND_EN,
        WBEM_E_DATABASE_VER_MISMATCH => WBEM_E_DATABASE_VER_MISMATCH_EN,
        WBEM_E_VETO_DELETE => WBEM_E_VETO_DELETE_EN,
        WBEM_E_VETO_PUT => WBEM_E_VETO_PUT_EN,
        WBEM_E_INVALID_LOCALE => WBEM_E_INVALID_LOCALE_EN,
        WBEM_E_PROVIDER_SUSPENDED => WBEM_E_PROVIDER_SUSPENDED_EN,
        WBEM_E_SYNCHRONIZATION_REQUIRED => WBEM_E_SYNCHRONIZATION_REQUIRED_EN,
        WBEM_E_NO_SCHEMA => WBEM_E_NO_SCHEMA_EN,
        WBEM_E_PROVIDER_ALREADY_REGISTERED => WBEM_E_PROVIDER_ALREADY_REGISTERED_EN,
        WBEM_E_PROVIDER_NOT_REGISTERED => WBEM_E_PROVIDER_NOT_REGISTERED_EN,
        WBEM_E_FATAL_TRANSPORT_ERROR => WBEM_E_FATAL_TRANSPORT_ERROR_EN,
        WBEM_E_ENCRYPTED_CONNECTION_REQUIRED => WBEM_E_ENCRYPTED_CONNECTION_REQUIRED_EN,
        WBEM_E_PROVIDER_TIMED_OUT => WBEM_E_PROVIDER_TIMED_OUT_EN,
        WBEM_E_NO_KEY => WBEM_E_NO_KEY_EN,
        WBEM_E_PROVIDER_DISABLED => WBEM_E_PROVIDER_DISABLED_EN,
        WBEMESS_E_REGISTRATION_TOO_BROAD => WBEMESS_E_REGISTRATION_TOO_BROAD_EN,
        WBEMESS_E_REGISTRATION_TOO_PRECISE => WBEMESS_E_REGISTRATION_TOO_PRECISE_EN,
        WBEMESS_E_AUTHZ_NOT_PRIVILEGED => WBEMESS_E_AUTHZ_NOT_PRIVILEGED_EN,
        WBEMMOF_E_EXPECTED_QUALIFIER_NAME => WBEMMOF_E_EXPECTED_QUALIFIER_NAME_EN,
        WBEMMOF_E_EXPECTED_SEMI => WBEMMOF_E_EXPECTED_SEMI_EN,
        WBEMMOF_E_EXPECTED_OPEN_BRACE => WBEMMOF_E_EXPECTED_OPEN_BRACE_EN,
        WBEMMOF_E_EXPECTED_CLOSE_BRACE => WBEMMOF_E_EXPECTED_CLOSE_BRACE_EN,
        WBEMMOF_E_EXPECTED_CLOSE_BRACKET => WBEMMOF_E_EXPECTED_CLOSE_BRACKET_EN,
        WBEMMOF_E_EXPECTED_CLOSE_PAREN => WBEMMOF_E_EXPECTED_CLOSE_PAREN_EN,
        WBEMMOF_E_ILLEGAL_CONSTANT_VALUE => WBEMMOF_E_ILLEGAL_CONSTANT_VALUE_EN,
        WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER => WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER_EN,
        WBEMMOF_E_EXPECTED_OPEN_PAREN => WBEMMOF_E_EXPECTED_OPEN_PAREN_EN,
        WBEMMOF_E_UNRECOGNIZED_TOKEN => WBEMMOF_E_UNRECOGNIZED_TOKEN_EN,
        WBEMMOF_E_UNRECOGNIZED_TYPE => WBEMMOF_E_UNRECOGNIZED_TYPE_EN,
        WBEMMOF_E_EXPECTED_PROPERTY_NAME => WBEMMOF_E_EXPECTED_PROPERTY_NAME_EN,
        WBEMMOF_E_TYPEDEF_NOT_SUPPORTED => WBEMMOF_E_TYPEDEF_NOT_SUPPORTED_EN,
        WBEMMOF_E_UNEXPECTED_ALIAS => WBEMMOF_E_UNEXPECTED_ALIAS_EN,
        WBEMMOF_E_UNEXPECTED_ARRAY_INIT => WBEMMOF_E_UNEXPECTED_ARRAY_INIT_EN,
        WBEMMOF_E_INVALID_AMENDMENT_SYNTAX => WBEMMOF_E_INVALID_AMENDMENT_SYNTAX_EN,
        WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT => WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT_EN,
        WBEMMOF_E_INVALID_PRAGMA => WBEMMOF_E_INVALID_PRAGMA_EN,
        WBEMMOF_E_INVALID_NAMESPACE_SYNTAX => WBEMMOF_E_INVALID_NAMESPACE_SYNTAX_EN,
        WBEMMOF_E_EXPECTED_CLASS_NAME => WBEMMOF_E_EXPECTED_CLASS_NAME_EN,
        WBEMMOF_E_TYPE_MISMATCH => WBEMMOF_E_TYPE_MISMATCH_EN,
        WBEMMOF_E_EXPECTED_ALIAS_NAME => WBEMMOF_E_EXPECTED_ALIAS_NAME_EN,
        WBEMMOF_E_INVALID_CLASS_DECLARATION => WBEMMOF_E_INVALID_CLASS_DECLARATION_EN,
        WBEMMOF_E_INVALID_INSTANCE_DECLARATION => WBEMMOF_E_INVALID_INSTANCE_DECLARATION_EN,
        WBEMMOF_E_EXPECTED_DOLLAR => WBEMMOF_E_EXPECTED_DOLLAR_EN,
        WBEMMOF_E_CIMTYPE_QUALIFIER => WBEMMOF_E_CIMTYPE_QUALIFIER_EN,
        WBEMMOF_E_DUPLICATE_PROPERTY => WBEMMOF_E_DUPLICATE_PROPERTY_EN,
        WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION => WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION_EN,
        WBEMMOF_E_OUT_OF_RANGE => WBEMMOF_E_OUT_OF_RANGE_EN,
        WBEMMOF_E_INVALID_FILE => WBEMMOF_E_INVALID_FILE_EN,
        WBEMMOF_E_ALIASES_IN_EMBEDDED => WBEMMOF_E_ALIASES_IN_EMBEDDED_EN,
        WBEMMOF_E_NULL_ARRAY_ELEM => WBEMMOF_E_NULL_ARRAY_ELEM_EN,
        WBEMMOF_E_DUPLICATE_QUALIFIER => WBEMMOF_E_DUPLICATE_QUALIFIER_EN,
        WBEMMOF_E_EXPECTED_FLAVOR_TYPE => WBEMMOF_E_EXPECTED_FLAVOR_TYPE_EN,
        WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES => WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES_EN,
        WBEMMOF_E_MULTIPLE_ALIASES => WBEMMOF_E_MULTIPLE_ALIASES_EN,
        WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2 => WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2_EN,
        WBEMMOF_E_NO_ARRAYS_RETURNED => WBEMMOF_E_NO_ARRAYS_RETURNED_EN,
        WBEMMOF_E_MUST_BE_IN_OR_OUT => WBEMMOF_E_MUST_BE_IN_OR_OUT_EN,
        WBEMMOF_E_INVALID_FLAGS_SYNTAX => WBEMMOF_E_INVALID_FLAGS_SYNTAX_EN,
        WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE => WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE_EN,
        WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE => WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE_EN,
        WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE => WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE_EN,
        WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX => WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX_EN,
        WBEMMOF_E_INVALID_QUALIFIER_SYNTAX => WBEMMOF_E_INVALID_QUALIFIER_SYNTAX_EN,
        WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE => WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE_EN,
        WBEMMOF_E_ERROR_CREATING_TEMP_FILE => WBEMMOF_E_ERROR_CREATING_TEMP_FILE_EN,
        WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE => WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE_EN,
        WBEMMOF_E_INVALID_DELETECLASS_SYNTAX => WBEMMOF_E_INVALID_DELETECLASS_SYNTAX_EN,
        _ => match WBEM_EXTRA_RETURN_CODES(hres) {
            WBEM_E_RETRY_LATER => WBEM_E_RETRY_LATER_EN,
            WBEM_E_RESOURCE_CONTENTION => WBEM_E_RESOURCE_CONTENTION_EN,
            _ => "",
        },
    }
}

// English descriptions of WBEM constants hard-coded from:
// https://docs.microsoft.com/en-us/windows/win32/wmisdk/wmi-error-constants
// https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/WmiSdk/wmi-error-constants.md

const WBEM_E_FAILED_EN: &str = "Call failed.";

const WBEM_E_NOT_FOUND_EN: &str = "Object cannot be found.";

const WBEM_E_ACCESS_DENIED_EN: &str =
    "Current user does not have permission to perform the action.";

const WBEM_E_PROVIDER_FAILURE_EN: &str =
    "Provider has failed at some time other than during initialization.";

const WBEM_E_TYPE_MISMATCH_EN: &str = "Type mismatch occurred.";

const WBEM_E_OUT_OF_MEMORY_EN: &str = "Not enough memory for the operation.";

const WBEM_E_INVALID_CONTEXT_EN: &str = "The IWbemContext object is not valid.";

const WBEM_E_INVALID_PARAMETER_EN: &str = "One of the parameters to the call is not correct.";

const WBEM_E_NOT_AVAILABLE_EN: &str =
    "Resource, typically a remote server, is not currently available.";

const WBEM_E_CRITICAL_ERROR_EN: &str =
    "Internal, critical, and unexpected error occurred. Report the error to Microsoft Technical \
    Support.";

const WBEM_E_INVALID_STREAM_EN: &str =
    "One or more network packets were corrupted during a remote session.";

const WBEM_E_NOT_SUPPORTED_EN: &str = "Feature or operation is not supported.";

const WBEM_E_INVALID_SUPERCLASS_EN: &str = "Parent class specified is not valid.";

const WBEM_E_INVALID_NAMESPACE_EN: &str = "Namespace specified cannot be found.";

const WBEM_E_INVALID_OBJECT_EN: &str = "Specified instance is not valid.";

const WBEM_E_INVALID_CLASS_EN: &str = "Specified class is not valid.";

const WBEM_E_PROVIDER_NOT_FOUND_EN: &str =
    "Provider referenced in the schema does not have a corresponding registration.";

const WBEM_E_INVALID_PROVIDER_REGISTRATION_EN: &str =
    "Provider referenced in the schema has an incorrect or incomplete registration.
    \n
    \nThis error may be caused by many conditions, including the following:
    \n
    \n• A missing #pragma namespace command in the Managed Object Format (MOF) file used to \
    register the provider. The provider may be registered in the wrong WMI namespace.
    \n• Failure to retrieve the COM registration.
    \n• Hosting model is not valid. For more information, see Provider Hosting and Security.
    \n• An class specified in the registration is not valid.
    \n• Failure to create an instance of or inherit from the __Win32Provider class to create the \
    provider registration in the MOF file.";

const WBEM_E_PROVIDER_LOAD_FAILURE_EN: &str =
    "COM cannot locate a provider referenced in the schema.
    \n
    \nThis error may be caused by many conditions, including the following:
    \n
    \n• Provider is using a WMI DLL that does not match the .lib file used when the provider was \
    built.
    \n• Provider's DLL, or any of the DLLs on which it depends, is corrupt.
    \n• Provider failed to export DllRegisterServer.
    \n• In-process provider was not registered using the regsvr32 command.
    \n• Out-of-process provider was not registered using the /regserver switch. For example, \
    myprog.exe /regserver.";

const WBEM_E_INITIALIZATION_FAILURE_EN: &str =
    "Component, such as a provider, failed to initialize for internal reasons.";

const WBEM_E_TRANSPORT_FAILURE_EN: &str =
    "Networking error that prevents normal operation has occurred.";

const WBEM_E_INVALID_OPERATION_EN: &str =
    "Requested operation is not valid. This error usually applies to invalid attempts to delete \
    classes or properties.";

const WBEM_E_INVALID_QUERY_EN: &str = "Query was not syntactically valid.";

const WBEM_E_INVALID_QUERY_TYPE_EN: &str = "Requested query language is not supported.";

const WBEM_E_ALREADY_EXISTS_EN: &str =
    "In a put operation, the wbemChangeFlagCreateOnly flag was specified, but the instance already \
    exists.";

const WBEM_E_OVERRIDE_NOT_ALLOWED_EN: &str =
    "Not possible to perform the add operation on this qualifier because the owning object does \
    not permit overrides.";

const WBEM_E_PROPAGATED_QUALIFIER_EN: &str =
    "User attempted to delete a qualifier that was not owned. The qualifier was inherited from a \
    parent class.";

const WBEM_E_PROPAGATED_PROPERTY_EN: &str =
    "User attempted to delete a property that was not owned. The property was inherited from a \
    parent class.";

const WBEM_E_UNEXPECTED_EN: &str =
    "Client made an unexpected and illegal sequence of calls, such as calling EndEnumeration \
    before calling BeginEnumeration.";

const WBEM_E_ILLEGAL_OPERATION_EN: &str =
    "User requested an illegal operation, such as spawning a class from an instance.";

const WBEM_E_CANNOT_BE_KEY_EN: &str =
    "Illegal attempt to specify a key qualifier on a property that cannot be a key. The keys are \
    specified in the class definition for an object and cannot be altered on a per-instance basis.";

const WBEM_E_INCOMPLETE_CLASS_EN: &str =
    "Current object is not a valid class definition. Either it is incomplete or it has not been \
    registered with WMI using SWbemObject.Put_.";

const WBEM_E_INVALID_SYNTAX_EN: &str = "Query is syntactically not valid.";

const WBEM_E_NONDECORATED_OBJECT_EN: &str = "Reserved for future use.";

const WBEM_E_READ_ONLY_EN: &str = "An attempt was made to modify a read-only property.";

const WBEM_E_PROVIDER_NOT_CAPABLE_EN: &str =
    "Provider cannot perform the requested operation. This can include a query that is too \
    complex, retrieving an instance, creating or updating a class, deleting a class, or \
    enumerating a class.";

const WBEM_E_CLASS_HAS_CHILDREN_EN: &str =
    "Attempt was made to make a change that invalidates a subclass.";

const WBEM_E_CLASS_HAS_INSTANCES_EN: &str =
    "Attempt was made to delete or modify a class that has instances.";

const WBEM_E_QUERY_NOT_IMPLEMENTED_EN: &str = "Reserved for future use.";

const WBEM_E_ILLEGAL_NULL_EN: &str =
    "Value of Nothing/NULL was specified for a property that must have a value, such as one that \
    is marked by a Key, Indexed, or Not_Null qualifier.";

const WBEM_E_INVALID_QUALIFIER_TYPE_EN: &str =
    "Variant value for a qualifier was provided that is not a legal qualifier type.";

const WBEM_E_INVALID_PROPERTY_TYPE_EN: &str = "CIM type specified for a property is not valid.";

const WBEM_E_VALUE_OUT_OF_RANGE_EN: &str =
    "Request was made with an out-of-range value or it is incompatible with the type.";

const WBEM_E_CANNOT_BE_SINGLETON_EN: &str =
    "Illegal attempt was made to make a class singleton, such as when the class is derived from a \
    non-singleton class.";

const WBEM_E_INVALID_CIM_TYPE_EN: &str = "CIM type specified is not valid.";

const WBEM_E_INVALID_METHOD_EN: &str = "Requested method is not available.";

const WBEM_E_INVALID_METHOD_PARAMETERS_EN: &str =
    "Parameters provided for the method are not valid.";

const WBEM_E_SYSTEM_PROPERTY_EN: &str =
    "There was an attempt to get qualifiers on a system property.";

const WBEM_E_INVALID_PROPERTY_EN: &str = "Property type is not recognized.";

const WBEM_E_CALL_CANCELLED_EN: &str =
    "Asynchronous process has been canceled internally or by the user. Note that due to the timing \
    and nature of the asynchronous operation, the operation may not have been truly canceled.";

const WBEM_E_SHUTTING_DOWN_EN: &str =
    "User has requested an operation while WMI is in the process of shutting down.";

const WBEM_E_PROPAGATED_METHOD_EN: &str =
    "Attempt was made to reuse an existing method name from a parent class and the signatures do \
    not match.";

const WBEM_E_UNSUPPORTED_PARAMETER_EN: &str =
    "One or more parameter values, such as a query text, is too complex or unsupported. WMI is \
    therefore requested to retry the operation with simpler parameters.";

const WBEM_E_MISSING_PARAMETER_ID_EN: &str = "Parameter was missing from the method call.";

const WBEM_E_INVALID_PARAMETER_ID_EN: &str =
    "Method parameter has an ID qualifier that is not valid.";

const WBEM_E_NONCONSECUTIVE_PARAMETER_IDS_EN: &str =
    "One or more of the method parameters have ID qualifiers that are out of sequence.";

const WBEM_E_PARAMETER_ID_ON_RETVAL_EN: &str = "Return value for a method has an ID qualifier.";

const WBEM_E_INVALID_OBJECT_PATH_EN: &str = "Specified object path was not valid.";

const WBEM_E_OUT_OF_DISK_SPACE_EN: &str =
    "Disk is out of space or the 4 GB limit on WMI repository (CIM repository) size is reached.";

const WBEM_E_BUFFER_TOO_SMALL_EN: &str =
    "Supplied buffer was too small to hold all of the objects in the enumerator or to read a \
    string property.";

const WBEM_E_UNSUPPORTED_PUT_EXTENSION_EN: &str =
    "Provider does not support the requested put operation.";

const WBEM_E_UNKNOWN_OBJECT_TYPE_EN: &str =
    "Object with an incorrect type or version was encountered during marshaling.";

const WBEM_E_UNKNOWN_PACKET_TYPE_EN: &str =
    "Packet with an incorrect type or version was encountered during marshaling.";

const WBEM_E_MARSHAL_VERSION_MISMATCH_EN: &str = "Packet has an unsupported version.";

const WBEM_E_MARSHAL_INVALID_SIGNATURE_EN: &str = "Packet appears to be corrupt.";

const WBEM_E_INVALID_QUALIFIER_EN: &str =
    "Attempt was made to mismatch qualifiers, such as putting [key] on an object instead of a \
    property.";

const WBEM_E_INVALID_DUPLICATE_PARAMETER_EN: &str =
    "Duplicate parameter was declared in a CIM method.";

const WBEM_E_TOO_MUCH_DATA_EN: &str = "Reserved for future use.";

const WBEM_E_SERVER_TOO_BUSY_EN: &str =
    "Call to IWbemObjectSink::Indicate has failed. The provider can refire the event.";

const WBEM_E_INVALID_FLAVOR_EN: &str = "Specified qualifier flavor was not valid.";

const WBEM_E_CIRCULAR_REFERENCE_EN: &str =
    "Attempt was made to create a reference that is circular (for example, deriving a class from \
    itself).";

const WBEM_E_UNSUPPORTED_CLASS_UPDATE_EN: &str = "Specified class is not supported.";

const WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE_EN: &str =
    "Attempt was made to change a key when instances or subclasses are already using the key.";

const WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE_EN: &str =
    "An attempt was made to change an index when instances or subclasses are already using the \
    index.";

const WBEM_E_TOO_MANY_PROPERTIES_EN: &str =
    "Attempt was made to create more properties than the current version of the class supports.";

const WBEM_E_UPDATE_TYPE_MISMATCH_EN: &str =
    "Property was redefined with a conflicting type in a derived class.";

const WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED_EN: &str =
    "Attempt was made in a derived class to override a qualifier that cannot be overridden.";

const WBEM_E_UPDATE_PROPAGATED_METHOD_EN: &str =
    "Method was re-declared with a conflicting signature in a derived class.";

const WBEM_E_METHOD_NOT_IMPLEMENTED_EN: &str =
    "Attempt was made to execute a method not marked with [implemented] in any relevant class.";

const WBEM_E_METHOD_DISABLED_EN: &str =
    "Attempt was made to execute a method marked with [disabled].";

const WBEM_E_REFRESHER_BUSY_EN: &str = "Refresher is busy with another operation.";

const WBEM_E_UNPARSABLE_QUERY_EN: &str = "Filtering query is syntactically not valid.";

const WBEM_E_NOT_EVENT_CLASS_EN: &str =
    "The FROM clause of a filtering query references a class that is not an event class (not \
    derived from __Event).";

const WBEM_E_MISSING_GROUP_WITHIN_EN: &str =
    "A GROUP BY clause was used without the corresponding GROUP WITHIN clause.";

const WBEM_E_MISSING_AGGREGATION_LIST_EN: &str =
    "A GROUP BY clause was used. Aggregation on all properties is not supported.";

const WBEM_E_PROPERTY_NOT_AN_OBJECT_EN: &str =
    "Dot notation was used on a property that is not an embedded object.";

const WBEM_E_AGGREGATING_BY_OBJECT_EN: &str =
    "A GROUP BY clause references a property that is an embedded object without using dot notation.";

const WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY_EN: &str =
    "Event provider registration query (__EventProviderRegistration) did not specify the classes \
    for which events were provided.";

const WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING_EN: &str =
    "Request was made to back up or restore the repository while it was in use by WinMgmt.exe, or \
    by the SVCHOST process that contains the WMI service.";

const WBEM_E_QUEUE_OVERFLOW_EN: &str =
    "Asynchronous delivery queue overflowed from the event consumer being too slow.";

const WBEM_E_PRIVILEGE_NOT_HELD_EN: &str =
    "Operation failed because the client did not have the necessary security privilege.";

const WBEM_E_INVALID_OPERATOR_EN: &str = "Operator is not valid for this property type.";

const WBEM_E_LOCAL_CREDENTIALS_EN: &str =
    "User specified a username/password/authority on a local connection. The user must use a blank \
    username/password and rely on default security.";

const WBEM_E_CANNOT_BE_ABSTRACT_EN: &str =
    "Class was made abstract when its parent class is not abstract.";

const WBEM_E_AMENDED_OBJECT_EN: &str =
    "Amended object was written without the WBEM_FLAG_USE_AMENDED_QUALIFIERS flag being specified.";

const WBEM_E_CLIENT_TOO_SLOW_EN: &str =
    "Client did not retrieve objects quickly enough from an enumeration. This constant is returned \
    when a client creates an enumeration object, but does not retrieve objects from the enumerator \
    in a timely fashion, causing the enumerator's object caches to back up.";

const WBEM_E_NULL_SECURITY_DESCRIPTOR_EN: &str = "Null security descriptor was used.";

const WBEM_E_TIMED_OUT_EN: &str = "Operation timed out.";

const WBEM_E_INVALID_ASSOCIATION_EN: &str = "Association is not valid.";

const WBEM_E_AMBIGUOUS_OPERATION_EN: &str = "Operation was ambiguous.";

const WBEM_E_QUOTA_VIOLATION_EN: &str =
    "WMI is taking up too much memory. This can be caused by low memory availability or excessive \
    memory consumption by WMI.";

const WBEM_E_TRANSACTION_CONFLICT: WBEMSTATUS = WBEMSTATUS(0x8004106D_u32 as i32);
const WBEM_E_TRANSACTION_CONFLICT_EN: &str = "Operation resulted in a transaction conflict.";

const WBEM_E_FORCED_ROLLBACK: WBEMSTATUS = WBEMSTATUS(0x8004106E_u32 as i32);
const WBEM_E_FORCED_ROLLBACK_EN: &str = "Transaction forced a rollback.";

const WBEM_E_UNSUPPORTED_LOCALE_EN: &str = "Locale used in the call is not supported.";

const WBEM_E_HANDLE_OUT_OF_DATE_EN: &str = "Object handle is out-of-date.";

const WBEM_E_CONNECTION_FAILED_EN: &str = "Connection to the SQL database failed.";

const WBEM_E_INVALID_HANDLE_REQUEST_EN: &str = "Handle request was not valid.";

const WBEM_E_PROPERTY_NAME_TOO_WIDE_EN: &str = "Property name contains more than 255 characters.";

const WBEM_E_CLASS_NAME_TOO_WIDE_EN: &str = "Class name contains more than 255 characters.";

const WBEM_E_METHOD_NAME_TOO_WIDE_EN: &str = "Method name contains more than 255 characters.";

const WBEM_E_QUALIFIER_NAME_TOO_WIDE_EN: &str = "Qualifier name contains more than 255 characters.";

const WBEM_E_RERUN_COMMAND_EN: &str =
    "The SQL command must be rerun because there is a deadlock in SQL. This can be returned only \
    when data is being stored in an SQL database.";

const WBEM_E_DATABASE_VER_MISMATCH_EN: &str =
    "The database version does not match the version that the repository driver processes.";

const WBEM_E_VETO_DELETE_EN: &str =
    "WMI cannot execute the delete operation because the provider does not allow it.";

const WBEM_E_VETO_PUT_EN: &str =
    "WMI cannot execute the put operation because the provider does not allow it.";

const WBEM_E_INVALID_LOCALE_EN: &str =
    "Specified locale identifier was not valid for the operation.";

const WBEM_E_PROVIDER_SUSPENDED_EN: &str = "Provider is suspended.";

const WBEM_E_SYNCHRONIZATION_REQUIRED_EN: &str =
    "Object must be written to the WMI repository and retrieved again before the requested \
    operation can succeed. This constant is returned when an object must be committed and \
    retrieved to see the property value.";

const WBEM_E_NO_SCHEMA_EN: &str = "Operation cannot be completed; no schema is available.";

const WBEM_E_PROVIDER_ALREADY_REGISTERED_EN: &str =
    "Provider cannot be registered because it is already registered.";

const WBEM_E_PROVIDER_NOT_REGISTERED_EN: &str = "Provider was not registered.";

const WBEM_E_FATAL_TRANSPORT_ERROR_EN: &str = "A fatal transport error occurred.";

const WBEM_E_ENCRYPTED_CONNECTION_REQUIRED_EN: &str =
    "User attempted to set a computer name or domain without an encrypted connection.";

const WBEM_E_PROVIDER_TIMED_OUT_EN: &str =
    "A provider failed to report results within the specified timeout.";

const WBEM_E_NO_KEY_EN: &str = "User attempted to put an instance with no defined key.";

const WBEM_E_PROVIDER_DISABLED_EN: &str =
    "User attempted to register a provider instance but the COM server for the provider instance \
    was unloaded.";

const WBEMESS_E_REGISTRATION_TOO_BROAD_EN: &str =
    "Provider registration overlaps with the system event domain.";

const WBEMESS_E_REGISTRATION_TOO_PRECISE_EN: &str = "A WITHIN clause was not used in this query.";

const WBEMESS_E_AUTHZ_NOT_PRIVILEGED_EN: &str =
    "This computer does not have the necessary domain permissions to support the security \
    functions that relate to the created subscription instance. Contact the Domain Administrator \
    to get this computer added to the Windows Authorization Access Group.";

const WBEM_E_RETRY_LATER_EN: &str = "Reserved for future use.";

const WBEM_E_RESOURCE_CONTENTION_EN: &str = "Reserved for future use.";

const WBEMMOF_E_EXPECTED_QUALIFIER_NAME_EN: &str = "Expected a qualifier name.";

const WBEMMOF_E_EXPECTED_SEMI_EN: &str = "Expected semicolon or '='.";

const WBEMMOF_E_EXPECTED_OPEN_BRACE_EN: &str = "Expected an opening brace.";

const WBEMMOF_E_EXPECTED_CLOSE_BRACE_EN: &str =
    "Missing closing brace or an illegal array element.";

const WBEMMOF_E_EXPECTED_CLOSE_BRACKET_EN: &str = "Expected a closing bracket.";

const WBEMMOF_E_EXPECTED_CLOSE_PAREN_EN: &str = "Expected closing parenthesis.";

const WBEMMOF_E_ILLEGAL_CONSTANT_VALUE_EN: &str =
    "Numeric value out of range or strings without quotes.";

const WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER_EN: &str = "Expected a type identifier.";

const WBEMMOF_E_EXPECTED_OPEN_PAREN_EN: &str = "Expected an open parenthesis.";

const WBEMMOF_E_UNRECOGNIZED_TOKEN_EN: &str = "Unexpected token in the file.";

const WBEMMOF_E_UNRECOGNIZED_TYPE_EN: &str = "Unrecognized or unsupported type identifier.";

const WBEMMOF_E_EXPECTED_PROPERTY_NAME_EN: &str = "Expected property or method name.";

const WBEMMOF_E_TYPEDEF_NOT_SUPPORTED_EN: &str = "Typedefs and enumerated types are not supported.";

const WBEMMOF_E_UNEXPECTED_ALIAS_EN: &str =
    "Only a reference to a class object can have an alias value.";

const WBEMMOF_E_UNEXPECTED_ARRAY_INIT_EN: &str =
    "Unexpected array initialization. Arrays must be declared with [].";

const WBEMMOF_E_INVALID_AMENDMENT_SYNTAX_EN: &str = "Namespace path syntax is not valid.";

const WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT_EN: &str = "Duplicate amendment specifiers.";

const WBEMMOF_E_INVALID_PRAGMA_EN: &str = "#pragma must be followed by a valid keyword.";

const WBEMMOF_E_INVALID_NAMESPACE_SYNTAX_EN: &str = "Namespace path syntax is not valid.";

const WBEMMOF_E_EXPECTED_CLASS_NAME_EN: &str =
    "Unexpected character in class name must be an identifier.";

const WBEMMOF_E_TYPE_MISMATCH_EN: &str =
    "The value specified cannot be made into the appropriate type.";

const WBEMMOF_E_EXPECTED_ALIAS_NAME_EN: &str =
    "Dollar sign must be followed by an alias name as an identifier.";

const WBEMMOF_E_INVALID_CLASS_DECLARATION_EN: &str = "Class declaration is not valid.";

const WBEMMOF_E_INVALID_INSTANCE_DECLARATION_EN: &str =
    "The instance declaration is not valid. It must start with \"instance of\"";

const WBEMMOF_E_EXPECTED_DOLLAR_EN: &str =
    "Expected dollar sign. An alias in the form \"$name\" must follow the \"as\" keyword.";

const WBEMMOF_E_CIMTYPE_QUALIFIER_EN: &str =
    "\"CIMTYPE\" qualifier cannot be specified directly in a MOF file. Use standard type notation.";

const WBEMMOF_E_DUPLICATE_PROPERTY_EN: &str = "Duplicate property name was found in the MOF.";

const WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION_EN: &str =
    "Namespace syntax is not valid. References to other servers are not allowed.";

const WBEMMOF_E_OUT_OF_RANGE_EN: &str = "Value out of range.";

const WBEMMOF_E_INVALID_FILE_EN: &str = "The file is not a valid text MOF file or binary MOF file.";

const WBEMMOF_E_ALIASES_IN_EMBEDDED_EN: &str = "Embedded objects cannot be aliases.";

const WBEMMOF_E_NULL_ARRAY_ELEM_EN: &str = "NULL elements in an array are not supported.";

const WBEMMOF_E_DUPLICATE_QUALIFIER_EN: &str = "Qualifier was used more than once on the object.";

const WBEMMOF_E_EXPECTED_FLAVOR_TYPE_EN: &str =
    "Expected a flavor type such as ToInstance, ToSubClass, EnableOverride, or DisableOverride.";

const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES_EN: &str =
    "Combining EnableOverride and DisableOverride on same qualifier is not legal.";

const WBEMMOF_E_MULTIPLE_ALIASES_EN: &str = "An alias cannot be used twice.";

const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2_EN: &str =
    "Combining Restricted, and ToInstance or ToSubClass is not legal.";

const WBEMMOF_E_NO_ARRAYS_RETURNED_EN: &str = "Methods cannot return array values.";

const WBEMMOF_E_MUST_BE_IN_OR_OUT_EN: &str = "Arguments must have an In or Out qualifier.";

const WBEMMOF_E_INVALID_FLAGS_SYNTAX_EN: &str = "Flags syntax is not valid.";

const WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE_EN: &str =
    "The final brace and semi-colon for a class are missing.";

const WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE_EN: &str =
    "A CIM version 2.2 feature is not supported for a qualifier value.";

const WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE_EN: &str =
    "The CIM version 2.2 data type is not supported.";

const WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX_EN: &str =
    "The delete instance syntax is not valid. It should be #pragma \
    DeleteInstance(\"instancepath\", FAIL|NOFAIL)";

const WBEMMOF_E_INVALID_QUALIFIER_SYNTAX_EN: &str =
    "The qualifier syntax is not valid. It should be \
    qualifiername:type=value,scope(class|instance), flavorname .";

const WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE_EN: &str =
    "The qualifier is used outside of its scope.";

const WBEMMOF_E_ERROR_CREATING_TEMP_FILE_EN: &str =
    "Error creating temporary file. The temporary file is an intermediate stage in the MOF \
    compilation.";

const WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE_EN: &str =
    "A file included in the MOF by the preprocessor command #include is not valid.";

const WBEMMOF_E_INVALID_DELETECLASS_SYNTAX_EN: &str =
    "The syntax for the preprocessor commands #pragma deleteinstance or #pragma deleteclass is not \
    valid.";
