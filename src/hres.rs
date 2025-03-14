pub const fn to_str(hres: i32) -> &'static str {
    match hres as u32 {
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
        // WBEM_E_METHOD_DISABLED => WBEM_E_METHOD_DISABLED_EN,
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
        WBEM_E_RETRY_LATER => WBEM_E_RETRY_LATER_EN,
        WBEM_E_RESOURCE_CONTENTION => WBEM_E_RESOURCE_CONTENTION_EN,
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
        // WBEMMOF_E_EXPECTED_PROPERTY_NAME => WBEMMOF_E_EXPECTED_PROPERTY_NAME_EN,
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
        x if x >= 0x80041068 && x <= 0x80041099 => "(WMI)",
        x if x >= 0x80070000 && x <= 0x80079999 => "(OS)",
        x if x >= 0x80040000 && x <= 0x80040999 => "(DCOM)",
        x if x >= 0x80050000 && x <= 0x80059999 => "(ADSI/LDAP)",
        _ => "(UNKNOWN)",
    }
}

// WBEM constants and English descriptions hard-coded from:
// https://docs.microsoft.com/en-us/windows/win32/wmisdk/wmi-error-constants
// https://github.com/MicrosoftDocs/win32/blob/docs/desktop-src/WmiSdk/wmi-error-constants.md

const WBEM_E_FAILED: u32 = 0x80041001;
const WBEM_E_FAILED_EN: &str = "(WBEM_E_FAILED) Call failed.";

const WBEM_E_NOT_FOUND: u32 = 0x80041002;
const WBEM_E_NOT_FOUND_EN: &str = "(WBEM_E_NOT_FOUND) Object cannot be found.";

const WBEM_E_ACCESS_DENIED: u32 = 0x80041003;
const WBEM_E_ACCESS_DENIED_EN: &str =
    "(WBEM_E_ACCESS_DENIED) Current user does not have permission to perform the action.";

const WBEM_E_PROVIDER_FAILURE: u32 = 0x80041004;
const WBEM_E_PROVIDER_FAILURE_EN: &str =
    "(WBEM_E_PROVIDER_FAILURE) Provider has failed at some time other than during initialization.";

const WBEM_E_TYPE_MISMATCH: u32 = 0x80041005;
const WBEM_E_TYPE_MISMATCH_EN: &str = "(WBEM_E_TYPE_MISMATCH) Type mismatch occurred.";

const WBEM_E_OUT_OF_MEMORY: u32 = 0x80041006;
const WBEM_E_OUT_OF_MEMORY_EN: &str = "(WBEM_E_OUT_OF_MEMORY) Not enough memory for the operation.";

const WBEM_E_INVALID_CONTEXT: u32 = 0x80041007;
const WBEM_E_INVALID_CONTEXT_EN: &str =
    "(WBEM_E_INVALID_CONTEXT) The IWbemContext object is not valid.";

const WBEM_E_INVALID_PARAMETER: u32 = 0x80041008;
const WBEM_E_INVALID_PARAMETER_EN: &str =
    "(WBEM_E_INVALID_PARAMETER) One of the parameters to the call is not correct.";

const WBEM_E_NOT_AVAILABLE: u32 = 0x80041009;
const WBEM_E_NOT_AVAILABLE_EN: &str =
    "(WBEM_E_NOT_AVAILABLE) Resource, typically a remote server, is not currently available.";

const WBEM_E_CRITICAL_ERROR: u32 = 0x8004100A;
const WBEM_E_CRITICAL_ERROR_EN: &str =
    "(WBEM_E_CRITICAL_ERROR) Internal, critical, and unexpected error occurred. Report the error \
    to Microsoft Technical Support.";

const WBEM_E_INVALID_STREAM: u32 = 0x8004100B;
const WBEM_E_INVALID_STREAM_EN: &str =
    "(WBEM_E_INVALID_STREAM) One or more network packets were corrupted during a remote session.";

const WBEM_E_NOT_SUPPORTED: u32 = 0x8004100C;
const WBEM_E_NOT_SUPPORTED_EN: &str =
    "(WBEM_E_NOT_SUPPORTED) Feature or operation is not supported.";

const WBEM_E_INVALID_SUPERCLASS: u32 = 0x8004100D;
const WBEM_E_INVALID_SUPERCLASS_EN: &str =
    "(WBEM_E_INVALID_SUPERCLASS) Parent class specified is not valid.";

const WBEM_E_INVALID_NAMESPACE: u32 = 0x8004100E;
const WBEM_E_INVALID_NAMESPACE_EN: &str =
    "(WBEM_E_INVALID_NAMESPACE) Namespace specified cannot be found.";

const WBEM_E_INVALID_OBJECT: u32 = 0x8004100F;
const WBEM_E_INVALID_OBJECT_EN: &str = "(WBEM_E_INVALID_OBJECT) Specified instance is not valid.";

const WBEM_E_INVALID_CLASS: u32 = 0x80041010;
const WBEM_E_INVALID_CLASS_EN: &str = "(WBEM_E_INVALID_CLASS) Specified class is not valid.";

const WBEM_E_PROVIDER_NOT_FOUND: u32 = 0x80041011;
const WBEM_E_PROVIDER_NOT_FOUND_EN: &str =
    "(WBEM_E_PROVIDER_NOT_FOUND) Provider referenced in the schema does not have a corresponding \
 registration.";

const WBEM_E_INVALID_PROVIDER_REGISTRATION: u32 = 2147749906;
const WBEM_E_INVALID_PROVIDER_REGISTRATION_EN: &str =
    "(WBEM_E_INVALID_PROVIDER_REGISTRATION) Provider referenced in the schema has an incorrect or \
    incomplete registration.
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

const WBEM_E_PROVIDER_LOAD_FAILURE: u32 = 0x80041013;
const WBEM_E_PROVIDER_LOAD_FAILURE_EN: &str =
    "(WBEM_E_PROVIDER_LOAD_FAILURE) COM cannot locate a provider referenced in the schema.
    \n
    \nThis error may be caused by many conditions, including the following:
    \n
    \n• Provider is using a WMI DLL that does not match the .lib file used when the provider was \
    built.
    \n• Provider's DLL, or any of the DLLs on which it depends, is corrupt.
    \n• Provider failed to export DllRegisterServer.
    \n• In-process provider was not registered using the regsvr32 command.
    \n• Out-of-process provider was not registered using the /regserver switch. For example,\
    myprog.exe /regserver.";

const WBEM_E_INITIALIZATION_FAILURE: u32 = 0x80041014;
const WBEM_E_INITIALIZATION_FAILURE_EN: &str =
 "(WBEM_E_INITIALIZATION_FAILURE) Component, such as a provider, failed to initialize for internal \
 reasons.";

const WBEM_E_TRANSPORT_FAILURE: u32 = 0x80041015;
const WBEM_E_TRANSPORT_FAILURE_EN: &str =
    "(WBEM_E_TRANSPORT_FAILURE) Networking error that prevents normal operation has occurred.";

const WBEM_E_INVALID_OPERATION: u32 = 0x80041016;
const WBEM_E_INVALID_OPERATION_EN: &str =
    "(WBEM_E_INVALID_OPERATION) Requested operation is not valid. This error usually applies to \
    invalid attempts to delete classes or properties.";

const WBEM_E_INVALID_QUERY: u32 = 0x80041017;
const WBEM_E_INVALID_QUERY_EN: &str = "(WBEM_E_INVALID_QUERY) Query was not syntactically valid.";

const WBEM_E_INVALID_QUERY_TYPE: u32 = 0x80041018;
const WBEM_E_INVALID_QUERY_TYPE_EN: &str =
    "(WBEM_E_INVALID_QUERY_TYPE) Requested query language is not supported.";

const WBEM_E_ALREADY_EXISTS: u32 = 0x80041019;
const WBEM_E_ALREADY_EXISTS_EN: &str =
    "(WBEM_E_ALREADY_EXISTS) In a put operation, the wbemChangeFlagCreateOnly flag was specified, \
    but the instance already exists.";

const WBEM_E_OVERRIDE_NOT_ALLOWED: u32 = 0x8004101A;
const WBEM_E_OVERRIDE_NOT_ALLOWED_EN: &str =
    "(WBEM_E_OVERRIDE_NOT_ALLOWED) Not possible to perform the add operation on this qualifier \
    because the owning object does not permit overrides.";

const WBEM_E_PROPAGATED_QUALIFIER: u32 = 0x8004101B;
const WBEM_E_PROPAGATED_QUALIFIER_EN: &str =
    "(WBEM_E_PROPAGATED_QUALIFIER) User attempted to delete a qualifier that was not owned. The \
    qualifier was inherited from a parent class.";

const WBEM_E_PROPAGATED_PROPERTY: u32 = 0x8004101C;
const WBEM_E_PROPAGATED_PROPERTY_EN: &str =
    "(WBEM_E_PROPAGATED_PROPERTY) User attempted to delete a property that was not owned. The \
    property was inherited from a parent class.";

const WBEM_E_UNEXPECTED: u32 = 0x8004101D;
const WBEM_E_UNEXPECTED_EN: &str =
    "(WBEM_E_UNEXPECTED) Client made an unexpected and illegal sequence of calls, such as calling \
    EndEnumeration before calling BeginEnumeration.";

const WBEM_E_ILLEGAL_OPERATION: u32 = 0x8004101E;
const WBEM_E_ILLEGAL_OPERATION_EN: &str =
    "(WBEM_E_ILLEGAL_OPERATION) User requested an illegal operation, such as spawning a class from \
    an instance.";

const WBEM_E_CANNOT_BE_KEY: u32 = 0x8004101F;
const WBEM_E_CANNOT_BE_KEY_EN: &str =
    "(WBEM_E_CANNOT_BE_KEY) Illegal attempt to specify a key qualifier on a property that cannot \
    be a key. The keys are specified in the class definition for an object and cannot be altered \
    on a per-instance basis.";

const WBEM_E_INCOMPLETE_CLASS: u32 = 0x80041020;
const WBEM_E_INCOMPLETE_CLASS_EN: &str =
    "(WBEM_E_INCOMPLETE_CLASS) Current object is not a valid class definition. Either it is \
    incomplete or it has not been registered with WMI using SWbemObject.Put_.";

const WBEM_E_INVALID_SYNTAX: u32 = 0x80041021;
const WBEM_E_INVALID_SYNTAX_EN: &str = "(WBEM_E_INVALID_SYNTAX) Query is syntactically not valid.";

const WBEM_E_NONDECORATED_OBJECT: u32 = 0x80041022;
const WBEM_E_NONDECORATED_OBJECT_EN: &str = "(WBEM_E_NONDECORATED_OBJECT) Reserved for future use.";

const WBEM_E_READ_ONLY: u32 = 0x80041023;
const WBEM_E_READ_ONLY_EN: &str =
    "(WBEM_E_READ_ONLY) An attempt was made to modify a read-only property.";

const WBEM_E_PROVIDER_NOT_CAPABLE: u32 = 0x80041024;
const WBEM_E_PROVIDER_NOT_CAPABLE_EN: &str =
    "(WBEM_E_PROVIDER_NOT_CAPABLE) Provider cannot perform the requested operation. This can \
    include a query that is too complex, retrieving an instance, creating or updating a class, \
    deleting a class, or enumerating a class.";

const WBEM_E_CLASS_HAS_CHILDREN: u32 = 0x80041025;
const WBEM_E_CLASS_HAS_CHILDREN_EN: &str =
    "(WBEM_E_CLASS_HAS_CHILDREN) Attempt was made to make a change that invalidates a subclass.";

const WBEM_E_CLASS_HAS_INSTANCES: u32 = 0x80041026;
const WBEM_E_CLASS_HAS_INSTANCES_EN: &str =
    "(WBEM_E_CLASS_HAS_INSTANCES) Attempt was made to delete or modify a class that has instances.";

const WBEM_E_QUERY_NOT_IMPLEMENTED: u32 = 0x80041027;
const WBEM_E_QUERY_NOT_IMPLEMENTED_EN: &str =
    "(WBEM_E_QUERY_NOT_IMPLEMENTED) Reserved for future use.";

const WBEM_E_ILLEGAL_NULL: u32 = 0x80041028;
const WBEM_E_ILLEGAL_NULL_EN: &str =
    "(WBEM_E_ILLEGAL_NULL) Value of Nothing/NULL was specified for a property that must have a \
    value, such as one that is marked by a Key, Indexed, or Not_Null qualifier.";

const WBEM_E_INVALID_QUALIFIER_TYPE: u32 = 0x80041029;
const WBEM_E_INVALID_QUALIFIER_TYPE_EN: &str =
    "(WBEM_E_INVALID_QUALIFIER_TYPE) Variant value for a qualifier was provided that is not a \
    legal qualifier type.";

const WBEM_E_INVALID_PROPERTY_TYPE: u32 = 0x8004102A;
const WBEM_E_INVALID_PROPERTY_TYPE_EN: &str =
    "(WBEM_E_INVALID_PROPERTY_TYPE) CIM type specified for a property is not valid.";

const WBEM_E_VALUE_OUT_OF_RANGE: u32 = 0x8004102B;
const WBEM_E_VALUE_OUT_OF_RANGE_EN: &str =
    "(WBEM_E_VALUE_OUT_OF_RANGE) Request was made with an out-of-range value or it is incompatible \
    with the type.";

const WBEM_E_CANNOT_BE_SINGLETON: u32 = 0x8004102C;
const WBEM_E_CANNOT_BE_SINGLETON_EN: &str =
    "(WBEM_E_CANNOT_BE_SINGLETON) Illegal attempt was made to make a class singleton, such as \
    when the class is derived from a non-singleton class.";

const WBEM_E_INVALID_CIM_TYPE: u32 = 0x8004102D;
const WBEM_E_INVALID_CIM_TYPE_EN: &str =
    "(WBEM_E_INVALID_CIM_TYPE) CIM type specified is not valid.";

const WBEM_E_INVALID_METHOD: u32 = 0x8004102E;
const WBEM_E_INVALID_METHOD_EN: &str = "(WBEM_E_INVALID_METHOD) Requested method is not available.";

const WBEM_E_INVALID_METHOD_PARAMETERS: u32 = 0x8004102F;
const WBEM_E_INVALID_METHOD_PARAMETERS_EN: &str =
    "(WBEM_E_INVALID_METHOD_PARAMETERS) Parameters provided for the method are not valid.";

const WBEM_E_SYSTEM_PROPERTY: u32 = 0x80041030;
const WBEM_E_SYSTEM_PROPERTY_EN: &str =
    "(WBEM_E_SYSTEM_PROPERTY) There was an attempt to get qualifiers on a system property.";

const WBEM_E_INVALID_PROPERTY: u32 = 0x80041031;
const WBEM_E_INVALID_PROPERTY_EN: &str =
    "(WBEM_E_INVALID_PROPERTY) Property type is not recognized.";

const WBEM_E_CALL_CANCELLED: u32 = 0x80041032;
const WBEM_E_CALL_CANCELLED_EN: &str =
    "(WBEM_E_CALL_CANCELLED) Asynchronous process has been canceled internally or by the user. \
    Note that due to the timing and nature of the asynchronous operation, the operation may not have been truly canceled.";

const WBEM_E_SHUTTING_DOWN: u32 = 0x80041033;
const WBEM_E_SHUTTING_DOWN_EN: &str =
    "(WBEM_E_SHUTTING_DOWN) User has requested an operation while WMI is in the process of \
    shutting down.";

const WBEM_E_PROPAGATED_METHOD: u32 = 0x80041034;
const WBEM_E_PROPAGATED_METHOD_EN: &str =
    "(WBEM_E_PROPAGATED_METHOD) Attempt was made to reuse an existing method name from a parent \
    class and the signatures do not match.";

const WBEM_E_UNSUPPORTED_PARAMETER: u32 = 0x80041035;
const WBEM_E_UNSUPPORTED_PARAMETER_EN: &str =
    "(WBEM_E_UNSUPPORTED_PARAMETER) One or more parameter values, such as a query text, is too \
    complex or unsupported. WMI is therefore requested to retry the operation with simpler \
    parameters.";

const WBEM_E_MISSING_PARAMETER_ID: u32 = 0x80041036;
const WBEM_E_MISSING_PARAMETER_ID_EN: &str =
    "(WBEM_E_MISSING_PARAMETER_ID) Parameter was missing from the method call.";

const WBEM_E_INVALID_PARAMETER_ID: u32 = 0x80041037;
const WBEM_E_INVALID_PARAMETER_ID_EN: &str =
    "(WBEM_E_INVALID_PARAMETER_ID) Method parameter has an ID qualifier that is not valid.";

const WBEM_E_NONCONSECUTIVE_PARAMETER_IDS: u32 = 0x80041038;
const WBEM_E_NONCONSECUTIVE_PARAMETER_IDS_EN: &str =
    "(WBEM_E_NONCONSECUTIVE_PARAMETER_IDS) One or more of the method parameters have ID \
    qualifiers that are out of sequence.";

const WBEM_E_PARAMETER_ID_ON_RETVAL: u32 = 0x80041039;
const WBEM_E_PARAMETER_ID_ON_RETVAL_EN: &str =
    "(WBEM_E_PARAMETER_ID_ON_RETVAL) Return value for a method has an ID qualifier.";

const WBEM_E_INVALID_OBJECT_PATH: u32 = 0x8004103A;
const WBEM_E_INVALID_OBJECT_PATH_EN: &str =
    "(WBEM_E_INVALID_OBJECT_PATH) Specified object path was not valid.";

const WBEM_E_OUT_OF_DISK_SPACE: u32 = 0x8004103B;
const WBEM_E_OUT_OF_DISK_SPACE_EN: &str =
    "(WBEM_E_OUT_OF_DISK_SPACE) Disk is out of space or the 4 GB limit on WMI repository (CIM \
    repository) size is reached.";

const WBEM_E_BUFFER_TOO_SMALL: u32 = 0x8004103C;
const WBEM_E_BUFFER_TOO_SMALL_EN: &str =
    "(WBEM_E_BUFFER_TOO_SMALL) Supplied buffer was too small to hold all of the objects in the \
    enumerator or to read a string property.";

const WBEM_E_UNSUPPORTED_PUT_EXTENSION: u32 = 0x8004103D;
const WBEM_E_UNSUPPORTED_PUT_EXTENSION_EN: &str =
    "(WBEM_E_UNSUPPORTED_PUT_EXTENSION) Provider does not support the requested put operation.";

const WBEM_E_UNKNOWN_OBJECT_TYPE: u32 = 0x8004103E;
const WBEM_E_UNKNOWN_OBJECT_TYPE_EN: &str =
    "(WBEM_E_UNKNOWN_OBJECT_TYPE) Object with an incorrect type or version was encountered during \
    marshaling.";

const WBEM_E_UNKNOWN_PACKET_TYPE: u32 = 0x8004103F;
const WBEM_E_UNKNOWN_PACKET_TYPE_EN: &str =
    "(WBEM_E_UNKNOWN_PACKET_TYPE) Packet with an incorrect type or version was encountered during \
    marshaling.";

const WBEM_E_MARSHAL_VERSION_MISMATCH: u32 = 0x80041040;
const WBEM_E_MARSHAL_VERSION_MISMATCH_EN: &str =
    "(WBEM_E_MARSHAL_VERSION_MISMATCH) Packet has an unsupported version.";

const WBEM_E_MARSHAL_INVALID_SIGNATURE: u32 = 0x80041041;
const WBEM_E_MARSHAL_INVALID_SIGNATURE_EN: &str =
    "(WBEM_E_MARSHAL_INVALID_SIGNATURE) Packet appears to be corrupt.";

const WBEM_E_INVALID_QUALIFIER: u32 = 0x80041042;
const WBEM_E_INVALID_QUALIFIER_EN: &str =
    "(WBEM_E_INVALID_QUALIFIER) Attempt was made to mismatch qualifiers, such as putting [key] on \
    an object instead of a property.";

const WBEM_E_INVALID_DUPLICATE_PARAMETER: u32 = 0x80041043;
const WBEM_E_INVALID_DUPLICATE_PARAMETER_EN: &str =
    "(WBEM_E_INVALID_DUPLICATE_PARAMETER) Duplicate parameter was declared in a CIM method.";

const WBEM_E_TOO_MUCH_DATA: u32 = 0x80041044;
const WBEM_E_TOO_MUCH_DATA_EN: &str = "(WBEM_E_TOO_MUCH_DATA) Reserved for future use.";

const WBEM_E_SERVER_TOO_BUSY: u32 = 0x80041045;
const WBEM_E_SERVER_TOO_BUSY_EN: &str =
    "(WBEM_E_SERVER_TOO_BUSY) Call to IWbemObjectSink::Indicate has failed. The provider can \
    refire the event.";

const WBEM_E_INVALID_FLAVOR: u32 = 0x80041046;
const WBEM_E_INVALID_FLAVOR_EN: &str =
    "(WBEM_E_INVALID_FLAVOR) Specified qualifier flavor was not valid.";

const WBEM_E_CIRCULAR_REFERENCE: u32 = 0x80041047;
const WBEM_E_CIRCULAR_REFERENCE_EN: &str =
    "(WBEM_E_CIRCULAR_REFERENCE) Attempt was made to create a reference that is circular (for \
    example, deriving a class from itself).";

const WBEM_E_UNSUPPORTED_CLASS_UPDATE: u32 = 0x80041048;
const WBEM_E_UNSUPPORTED_CLASS_UPDATE_EN: &str =
    "(WBEM_E_UNSUPPORTED_CLASS_UPDATE) Specified class is not supported.";

const WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE: u32 = 0x80041049;
const WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE_EN: &str =
    "(WBEM_E_CANNOT_CHANGE_KEY_INHERITANCE) Attempt was made to change a key when instances or \
    subclasses are already using the key.";

const WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE: u32 = 0x80041050;
const WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE_EN: &str =
    "(WBEM_E_CANNOT_CHANGE_INDEX_INHERITANCE) An attempt was made to change an index when \
    instances or subclasses are already using the index.";

const WBEM_E_TOO_MANY_PROPERTIES: u32 = 0x80041051;
const WBEM_E_TOO_MANY_PROPERTIES_EN: &str =
    "(WBEM_E_TOO_MANY_PROPERTIES) Attempt was made to create more properties than the current \
    version of the class supports.";

const WBEM_E_UPDATE_TYPE_MISMATCH: u32 = 0x80041052;
const WBEM_E_UPDATE_TYPE_MISMATCH_EN: &str =
    "(WBEM_E_UPDATE_TYPE_MISMATCH) Property was redefined with a conflicting type in a derived \
    class.";

const WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED: u32 = 0x80041053;
const WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED_EN: &str =
    "(WBEM_E_UPDATE_OVERRIDE_NOT_ALLOWED) Attempt was made in a derived class to override a \
    qualifier that cannot be overridden.";

const WBEM_E_UPDATE_PROPAGATED_METHOD: u32 = 0x80041054;
const WBEM_E_UPDATE_PROPAGATED_METHOD_EN: &str =
    "(WBEM_E_UPDATE_PROPAGATED_METHOD) Method was re-declared with a conflicting signature in a \
    derived class.";

const WBEM_E_METHOD_NOT_IMPLEMENTED: u32 = 0x80041055;
const WBEM_E_METHOD_NOT_IMPLEMENTED_EN: &str =
    "(WBEM_E_METHOD_NOT_IMPLEMENTED) Attempt was made to execute a method not marked with \
    [implemented] in any relevant class.";

// const WBEM_E_METHOD_DISABLED: u32 = ??
// const WBEM_E_METHOD_DISABLED_EN: &str = "Attempt was made to execute a method marked with [disabled].";

const WBEM_E_REFRESHER_BUSY: u32 = 0x80041057;
const WBEM_E_REFRESHER_BUSY_EN: &str =
    "(WBEM_E_REFRESHER_BUSY) Refresher is busy with another operation.";

const WBEM_E_UNPARSABLE_QUERY: u32 = 0x80041058;
const WBEM_E_UNPARSABLE_QUERY_EN: &str =
    "(WBEM_E_UNPARSABLE_QUERY) Filtering query is syntactically not valid.";

const WBEM_E_NOT_EVENT_CLASS: u32 = 0x80041059;
const WBEM_E_NOT_EVENT_CLASS_EN: &str =
    "(WBEM_E_NOT_EVENT_CLASS) The FROM clause of a filtering query references a class that is not \
    an event class (not derived from __Event).";

const WBEM_E_MISSING_GROUP_WITHIN: u32 = 0x8004105A;
const WBEM_E_MISSING_GROUP_WITHIN_EN: &str =
    "(WBEM_E_MISSING_GROUP_WITHIN) A GROUP BY clause was used without the corresponding GROUP \
    WITHIN clause.";

const WBEM_E_MISSING_AGGREGATION_LIST: u32 = 0x8004105B;
const WBEM_E_MISSING_AGGREGATION_LIST_EN: &str =
    "(WBEM_E_MISSING_AGGREGATION_LIST) A GROUP BY clause was used. Aggregation on all properties \
    is not supported.";

const WBEM_E_PROPERTY_NOT_AN_OBJECT: u32 = 0x8004105C;
const WBEM_E_PROPERTY_NOT_AN_OBJECT_EN: &str =
    "(WBEM_E_PROPERTY_NOT_AN_OBJECT) Dot notation was used on a property that is not an embedded \
    object.";

const WBEM_E_AGGREGATING_BY_OBJECT: u32 = 0x8004105D;
const WBEM_E_AGGREGATING_BY_OBJECT_EN: &str =
    "(WBEM_E_AGGREGATING_BY_OBJECT) A GROUP BY clause references a property that is an embedded \
    object without using dot notation.";

const WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY: u32 = 0x8004105F;
const WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY_EN: &str =
    "(WBEM_E_UNINTERPRETABLE_PROVIDER_QUERY) Event provider registration query \
    (__EventProviderRegistration) did not specify the classes for which events were provided.";

const WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING: u32 = 0x80041060;
const WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING_EN: &str =
    "(WBEM_E_BACKUP_RESTORE_WINMGMT_RUNNING) Request was made to back up or restore the repository \
    while it was in use by WinMgmt.exe, or by the SVCHOST process that contains the WMI service.";

const WBEM_E_QUEUE_OVERFLOW: u32 = 0x80041061;
const WBEM_E_QUEUE_OVERFLOW_EN: &str =
    "(WBEM_E_QUEUE_OVERFLOW) Asynchronous delivery queue overflowed from the event consumer being \
    too slow.";

const WBEM_E_PRIVILEGE_NOT_HELD: u32 = 0x80041062;
const WBEM_E_PRIVILEGE_NOT_HELD_EN: &str =
    "(WBEM_E_PRIVILEGE_NOT_HELD) Operation failed because the client did not have the necessary \
    security privilege.";

const WBEM_E_INVALID_OPERATOR: u32 = 0x80041063;
const WBEM_E_INVALID_OPERATOR_EN: &str =
    "(WBEM_E_INVALID_OPERATOR) Operator is not valid for this property type.";

const WBEM_E_LOCAL_CREDENTIALS: u32 = 0x80041064;
const WBEM_E_LOCAL_CREDENTIALS_EN: &str =
    "(WBEM_E_LOCAL_CREDENTIALS) User specified a username/password/authority on a local \
    connection. The user must use a blank username/password and rely on default security.";

const WBEM_E_CANNOT_BE_ABSTRACT: u32 = 0x80041065;
const WBEM_E_CANNOT_BE_ABSTRACT_EN: &str =
    "(WBEM_E_CANNOT_BE_ABSTRACT) Class was made abstract when its parent class is not abstract.";

const WBEM_E_AMENDED_OBJECT: u32 = 0x80041066;
const WBEM_E_AMENDED_OBJECT_EN: &str =
    "(WBEM_E_AMENDED_OBJECT) Amended object was written without the \
    WBEM_FLAG_USE_AMENDED_QUALIFIERS flag being specified.";

const WBEM_E_CLIENT_TOO_SLOW: u32 = 0x80041067;
const WBEM_E_CLIENT_TOO_SLOW_EN: &str =
    "(WBEM_E_CLIENT_TOO_SLOW) Client did not retrieve objects quickly enough from an enumeration. \
    This constant is returned when a client creates an enumeration object, but does not retrieve \
    objects from the enumerator in a timely fashion, causing the enumerator's object caches to \
    back up.";

const WBEM_E_NULL_SECURITY_DESCRIPTOR: u32 = 0x80041068;
const WBEM_E_NULL_SECURITY_DESCRIPTOR_EN: &str =
    "(WBEM_E_NULL_SECURITY_DESCRIPTOR) Null security descriptor was used.";

const WBEM_E_TIMED_OUT: u32 = 0x80041069;
const WBEM_E_TIMED_OUT_EN: &str = "(WBEM_E_TIMED_OUT) Operation timed out.";

const WBEM_E_INVALID_ASSOCIATION: u32 = 2147749994;
const WBEM_E_INVALID_ASSOCIATION_EN: &str = "Association is not valid.";

const WBEM_E_AMBIGUOUS_OPERATION: u32 = 0x8004106B;
const WBEM_E_AMBIGUOUS_OPERATION_EN: &str = "(WBEM_E_AMBIGUOUS_OPERATION) Operation was ambiguous.";

const WBEM_E_QUOTA_VIOLATION: u32 = 0x8004106C;
const WBEM_E_QUOTA_VIOLATION_EN: &str =
    "(WBEM_E_QUOTA_VIOLATION) WMI is taking up too much memory. This can be caused by low memory \
    availability or excessive memory consumption by WMI.";

const WBEM_E_TRANSACTION_CONFLICT: u32 = 0x8004106D;
const WBEM_E_TRANSACTION_CONFLICT_EN: &str =
    "(WBEM_E_TRANSACTION_CONFLICT) Operation resulted in a transaction conflict.";

const WBEM_E_FORCED_ROLLBACK: u32 = 0x8004106E;
const WBEM_E_FORCED_ROLLBACK_EN: &str = "(WBEM_E_FORCED_ROLLBACK) Transaction forced a rollback.";

const WBEM_E_UNSUPPORTED_LOCALE: u32 = 0x8004106F;
const WBEM_E_UNSUPPORTED_LOCALE_EN: &str =
    "(WBEM_E_UNSUPPORTED_LOCALE) Locale used in the call is not supported.";

const WBEM_E_HANDLE_OUT_OF_DATE: u32 = 0x80041070;
const WBEM_E_HANDLE_OUT_OF_DATE_EN: &str =
    "(WBEM_E_HANDLE_OUT_OF_DATE) Object handle is out-of-date.";

const WBEM_E_CONNECTION_FAILED: u32 = 0x80041071;
const WBEM_E_CONNECTION_FAILED_EN: &str =
    "(WBEM_E_CONNECTION_FAILED) Connection to the SQL database failed.";

const WBEM_E_INVALID_HANDLE_REQUEST: u32 = 0x80041072;
const WBEM_E_INVALID_HANDLE_REQUEST_EN: &str =
    "(WBEM_E_INVALID_HANDLE_REQUEST) Handle request was not valid.";

const WBEM_E_PROPERTY_NAME_TOO_WIDE: u32 = 0x80041073;
const WBEM_E_PROPERTY_NAME_TOO_WIDE_EN: &str =
    "(WBEM_E_PROPERTY_NAME_TOO_WIDE) Property name contains more than 255 characters.";

const WBEM_E_CLASS_NAME_TOO_WIDE: u32 = 0x80041074;
const WBEM_E_CLASS_NAME_TOO_WIDE_EN: &str =
    "(WBEM_E_CLASS_NAME_TOO_WIDE) Class name contains more than 255 characters.";

const WBEM_E_METHOD_NAME_TOO_WIDE: u32 = 0x80041075;
const WBEM_E_METHOD_NAME_TOO_WIDE_EN: &str =
    "(WBEM_E_METHOD_NAME_TOO_WIDE) Method name contains more than 255 characters.";

const WBEM_E_QUALIFIER_NAME_TOO_WIDE: u32 = 0x80041076;
const WBEM_E_QUALIFIER_NAME_TOO_WIDE_EN: &str =
    "(WBEM_E_QUALIFIER_NAME_TOO_WIDE) Qualifier name contains more than 255 characters.";

const WBEM_E_RERUN_COMMAND: u32 = 0x80041077;
const WBEM_E_RERUN_COMMAND_EN: &str =
    "(WBEM_E_RERUN_COMMAND) The SQL command must be rerun because there is a deadlock in SQL. This \
    can be returned only when data is being stored in an SQL database.";

const WBEM_E_DATABASE_VER_MISMATCH: u32 = 0x80041078;
const WBEM_E_DATABASE_VER_MISMATCH_EN: &str =
    "(WBEM_E_DATABASE_VER_MISMATCH) The database version does not match the version that the \
    repository driver processes.";

const WBEM_E_VETO_DELETE: u32 = 0x80041079;
const WBEM_E_VETO_DELETE_EN: &str =
    "(WBEM_E_VETO_DELETE) WMI cannot execute the delete operation because the provider does not \
    allow it.";

const WBEM_E_VETO_PUT: u32 = 0x8004107A;
const WBEM_E_VETO_PUT_EN: &str = "(WBEM_E_VETO_PUT) WMI cannot execute the put operation because \
the provider does not allow it.";

const WBEM_E_INVALID_LOCALE: u32 = 0x80041080;
const WBEM_E_INVALID_LOCALE_EN: &str =
    "(WBEM_E_INVALID_LOCALE) Specified locale identifier was not valid for the operation.";

const WBEM_E_PROVIDER_SUSPENDED: u32 = 0x80041081;
const WBEM_E_PROVIDER_SUSPENDED_EN: &str = "(WBEM_E_PROVIDER_SUSPENDED) Provider is suspended.";

const WBEM_E_SYNCHRONIZATION_REQUIRED: u32 = 0x80041082;
const WBEM_E_SYNCHRONIZATION_REQUIRED_EN: &str =
    "(WBEM_E_SYNCHRONIZATION_REQUIRED) Object must be written to the WMI repository and retrieved \
    again before the requested operation can succeed. This constant is returned when an object \
    must be committed and retrieved to see the property value.";

const WBEM_E_NO_SCHEMA: u32 = 0x80041083;
const WBEM_E_NO_SCHEMA_EN: &str =
    "(WBEM_E_NO_SCHEMA) Operation cannot be completed; no schema is available.";

const WBEM_E_PROVIDER_ALREADY_REGISTERED: u32 = 0x119FD010;
const WBEM_E_PROVIDER_ALREADY_REGISTERED_EN: &str =
    "(WBEM_E_PROVIDER_ALREADY_REGISTERED) Provider cannot be registered because it is already\
     registered.";

const WBEM_E_PROVIDER_NOT_REGISTERED: u32 = 0x80041085;
const WBEM_E_PROVIDER_NOT_REGISTERED_EN: &str =
    "(WBEM_E_PROVIDER_NOT_REGISTERED) Provider was not registered.";

const WBEM_E_FATAL_TRANSPORT_ERROR: u32 = 0x80041086;
const WBEM_E_FATAL_TRANSPORT_ERROR_EN: &str =
    "(WBEM_E_FATAL_TRANSPORT_ERROR) A fatal transport error occurred.";

const WBEM_E_ENCRYPTED_CONNECTION_REQUIRED: u32 = 0x80041087;
const WBEM_E_ENCRYPTED_CONNECTION_REQUIRED_EN: &str =
    "(WBEM_E_ENCRYPTED_CONNECTION_REQUIRED) User attempted to set a computer name or domain \
    without an encrypted connection.";

const WBEM_E_PROVIDER_TIMED_OUT: u32 = 0x80041088;
const WBEM_E_PROVIDER_TIMED_OUT_EN: &str =
    "(WBEM_E_PROVIDER_TIMED_OUT) A provider failed to report results within the specified timeout.";

const WBEM_E_NO_KEY: u32 = 0x80041089;
const WBEM_E_NO_KEY_EN: &str =
    "(WBEM_E_NO_KEY) User attempted to put an instance with no defined key.";

const WBEM_E_PROVIDER_DISABLED: u32 = 0x8004108A;
const WBEM_E_PROVIDER_DISABLED_EN: &str =
    "(WBEM_E_PROVIDER_DISABLED) User attempted to register a provider instance but the COM server \
    for the provider instance was unloaded.";

const WBEMESS_E_REGISTRATION_TOO_BROAD: u32 = 0x80042001;
const WBEMESS_E_REGISTRATION_TOO_BROAD_EN: &str =
    "(WBEMESS_E_REGISTRATION_TOO_BROAD) Provider registration overlaps with the system event domain.";

const WBEMESS_E_REGISTRATION_TOO_PRECISE: u32 = 0x80042002;
const WBEMESS_E_REGISTRATION_TOO_PRECISE_EN: &str =
    "(WBEMESS_E_REGISTRATION_TOO_PRECISE) A WITHIN clause was not used in this query.";

const WBEMESS_E_AUTHZ_NOT_PRIVILEGED: u32 = 0x80042003;
const WBEMESS_E_AUTHZ_NOT_PRIVILEGED_EN: &str =
    "(WBEMESS_E_AUTHZ_NOT_PRIVILEGED) This computer does not have the necessary domain permissions \
    to support the security functions that relate to the created subscription instance. Contact \
    the Domain Administrator to get this computer added to the Windows Authorization Access Group.";

const WBEM_E_RETRY_LATER: u32 = 0x80043001;
const WBEM_E_RETRY_LATER_EN: &str = "(WBEM_E_RETRY_LATER) Reserved for future use.";

const WBEM_E_RESOURCE_CONTENTION: u32 = 0x80043002;
const WBEM_E_RESOURCE_CONTENTION_EN: &str = "(WBEM_E_RESOURCE_CONTENTION) Reserved for future use.";

const WBEMMOF_E_EXPECTED_QUALIFIER_NAME: u32 = 0x80044001;
const WBEMMOF_E_EXPECTED_QUALIFIER_NAME_EN: &str =
    "(WBEMMOF_E_EXPECTED_QUALIFIER_NAME) Expected a qualifier name.";

const WBEMMOF_E_EXPECTED_SEMI: u32 = 0x80044002;
const WBEMMOF_E_EXPECTED_SEMI_EN: &str = "(WBEMMOF_E_EXPECTED_SEMI) Expected semicolon or '='.";

const WBEMMOF_E_EXPECTED_OPEN_BRACE: u32 = 0x80044003;
const WBEMMOF_E_EXPECTED_OPEN_BRACE_EN: &str =
    "(WBEMMOF_E_EXPECTED_OPEN_BRACE) Expected an opening brace.";

const WBEMMOF_E_EXPECTED_CLOSE_BRACE: u32 = 0x80044004;
const WBEMMOF_E_EXPECTED_CLOSE_BRACE_EN: &str =
    "(WBEMMOF_E_EXPECTED_CLOSE_BRACE) Missing closing brace or an illegal array element.";

const WBEMMOF_E_EXPECTED_CLOSE_BRACKET: u32 = 0x80044005;
const WBEMMOF_E_EXPECTED_CLOSE_BRACKET_EN: &str =
    "(WBEMMOF_E_EXPECTED_CLOSE_BRACKET) Expected a closing bracket.";

const WBEMMOF_E_EXPECTED_CLOSE_PAREN: u32 = 0x80044006;
const WBEMMOF_E_EXPECTED_CLOSE_PAREN_EN: &str =
    "(WBEMMOF_E_EXPECTED_CLOSE_PAREN) Expected closing parenthesis.";

const WBEMMOF_E_ILLEGAL_CONSTANT_VALUE: u32 = 0x80044007;
const WBEMMOF_E_ILLEGAL_CONSTANT_VALUE_EN: &str =
    "(WBEMMOF_E_ILLEGAL_CONSTANT_VALUE) Numeric value out of range or strings without quotes.";

const WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER: u32 = 0x80044008;
const WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER_EN: &str =
    "(WBEMMOF_E_EXPECTED_TYPE_IDENTIFIER) Expected a type identifier.";

const WBEMMOF_E_EXPECTED_OPEN_PAREN: u32 = 0x80044009;
const WBEMMOF_E_EXPECTED_OPEN_PAREN_EN: &str =
    "(WBEMMOF_E_EXPECTED_OPEN_PAREN) Expected an open parenthesis.";

const WBEMMOF_E_UNRECOGNIZED_TOKEN: u32 = 0x8004400A;
const WBEMMOF_E_UNRECOGNIZED_TOKEN_EN: &str =
    "(WBEMMOF_E_UNRECOGNIZED_TOKEN) Unexpected token in the file.";

const WBEMMOF_E_UNRECOGNIZED_TYPE: u32 = 0x8004400B;
const WBEMMOF_E_UNRECOGNIZED_TYPE_EN: &str =
    "(WBEMMOF_E_UNRECOGNIZED_TYPE) Unrecognized or unsupported type identifier.";

// const WBEMMOF_E_EXPECTED_PROPERTY_NAME: u32 = 0x8004400B;
// const WBEMMOF_E_EXPECTED_PROPERTY_NAME_EN: &str =
//     "(WBEMMOF_E_EXPECTED_PROPERTY_NAME) Expected property or method name.";

const WBEMMOF_E_TYPEDEF_NOT_SUPPORTED: u32 = 0x8004400D;
const WBEMMOF_E_TYPEDEF_NOT_SUPPORTED_EN: &str =
    "(WBEMMOF_E_TYPEDEF_NOT_SUPPORTED) Typedefs and enumerated types are not supported.";

const WBEMMOF_E_UNEXPECTED_ALIAS: u32 = 0x8004400E;
const WBEMMOF_E_UNEXPECTED_ALIAS_EN: &str =
    "(WBEMMOF_E_UNEXPECTED_ALIAS) Only a reference to a class object can have an alias value.";

const WBEMMOF_E_UNEXPECTED_ARRAY_INIT: u32 = 0x8004400F;
const WBEMMOF_E_UNEXPECTED_ARRAY_INIT_EN: &str =
    "(WBEMMOF_E_UNEXPECTED_ARRAY_INIT) Unexpected array initialization. Arrays must be declared \
    with [].";

const WBEMMOF_E_INVALID_AMENDMENT_SYNTAX: u32 = 0x80044010;
const WBEMMOF_E_INVALID_AMENDMENT_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_AMENDMENT_SYNTAX) Namespace path syntax is not valid.";

const WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT: u32 = 0x80044011;
const WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT_EN: &str =
    "(WBEMMOF_E_INVALID_DUPLICATE_AMENDMENT) Duplicate amendment specifiers.";

const WBEMMOF_E_INVALID_PRAGMA: u32 = 0x80044012;
const WBEMMOF_E_INVALID_PRAGMA_EN: &str =
    "(WBEMMOF_E_INVALID_PRAGMA) #pragma must be followed by a valid keyword.";

const WBEMMOF_E_INVALID_NAMESPACE_SYNTAX: u32 = 0x80044013;
const WBEMMOF_E_INVALID_NAMESPACE_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_NAMESPACE_SYNTAX) Namespace path syntax is not valid.";

const WBEMMOF_E_EXPECTED_CLASS_NAME: u32 = 0x80044014;
const WBEMMOF_E_EXPECTED_CLASS_NAME_EN: &str =
    "(WBEMMOF_E_EXPECTED_CLASS_NAME) Unexpected character in class name must be an identifier.";

const WBEMMOF_E_TYPE_MISMATCH: u32 = 0x80044015;
const WBEMMOF_E_TYPE_MISMATCH_EN: &str =
    "(WBEMMOF_E_TYPE_MISMATCH) The value specified cannot be made into the appropriate type.";

const WBEMMOF_E_EXPECTED_ALIAS_NAME: u32 = 0x80044016;
const WBEMMOF_E_EXPECTED_ALIAS_NAME_EN: &str =
    "(WBEMMOF_E_EXPECTED_ALIAS_NAME) Dollar sign must be followed by an alias name as an identifier.";

const WBEMMOF_E_INVALID_CLASS_DECLARATION: u32 = 0x80044017;
const WBEMMOF_E_INVALID_CLASS_DECLARATION_EN: &str =
    "(WBEMMOF_E_INVALID_CLASS_DECLARATION) Class declaration is not valid.";

const WBEMMOF_E_INVALID_INSTANCE_DECLARATION: u32 = 0x80044018;
const WBEMMOF_E_INVALID_INSTANCE_DECLARATION_EN: &str =
    "(WBEMMOF_E_INVALID_INSTANCE_DECLARATION) The instance declaration is not valid. It must start \
    with \"instance of\"";

const WBEMMOF_E_EXPECTED_DOLLAR: u32 = 0x80044019;
const WBEMMOF_E_EXPECTED_DOLLAR_EN: &str =
    "(WBEMMOF_E_EXPECTED_DOLLAR) Expected dollar sign. An alias in the form \"$name\" must follow \
    the \"as\" keyword.";

const WBEMMOF_E_CIMTYPE_QUALIFIER: u32 = 0x8004401A;
const WBEMMOF_E_CIMTYPE_QUALIFIER_EN: &str =
    "(WBEMMOF_E_CIMTYPE_QUALIFIER) \"CIMTYPE\" qualifier cannot be specified directly in a MOF \
    file. Use standard type notation.";

const WBEMMOF_E_DUPLICATE_PROPERTY: u32 = 0x8004401B;
const WBEMMOF_E_DUPLICATE_PROPERTY_EN: &str =
    "(WBEMMOF_E_DUPLICATE_PROPERTY) Duplicate property name was found in the MOF.";

const WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION: u32 = 0x8004401C;
const WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION_EN: &str =
    "(WBEMMOF_E_INVALID_NAMESPACE_SPECIFICATION) Namespace syntax is not valid. References to \
    other servers are not allowed.";

const WBEMMOF_E_OUT_OF_RANGE: u32 = 0x8004401D;
const WBEMMOF_E_OUT_OF_RANGE_EN: &str = "(WBEMMOF_E_OUT_OF_RANGE) Value out of range.";

const WBEMMOF_E_INVALID_FILE: u32 = 0x8004401E;
const WBEMMOF_E_INVALID_FILE_EN: &str =
    "(WBEMMOF_E_INVALID_FILE) The file is not a valid text MOF file or binary MOF file.";

const WBEMMOF_E_ALIASES_IN_EMBEDDED: u32 = 0x8004401F;
const WBEMMOF_E_ALIASES_IN_EMBEDDED_EN: &str =
    "(WBEMMOF_E_ALIASES_IN_EMBEDDED) Embedded objects cannot be aliases.";

const WBEMMOF_E_NULL_ARRAY_ELEM: u32 = 0x80044020;
const WBEMMOF_E_NULL_ARRAY_ELEM_EN: &str =
    "(WBEMMOF_E_NULL_ARRAY_ELEM) NULL elements in an array are not supported.";

const WBEMMOF_E_DUPLICATE_QUALIFIER: u32 = 0x80044021;
const WBEMMOF_E_DUPLICATE_QUALIFIER_EN: &str =
    "(WBEMMOF_E_DUPLICATE_QUALIFIER) Qualifier was used more than once on the object.";

const WBEMMOF_E_EXPECTED_FLAVOR_TYPE: u32 = 0x80044022;
const WBEMMOF_E_EXPECTED_FLAVOR_TYPE_EN: &str =
    "(WBEMMOF_E_EXPECTED_FLAVOR_TYPE) Expected a flavor type such as ToInstance, ToSubClass, \
    EnableOverride, or DisableOverride.";

const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES: u32 = 0x80044023;
const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES_EN: &str =
    "(WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES) Combining EnableOverride and DisableOverride on same \
    qualifier is not legal.";

const WBEMMOF_E_MULTIPLE_ALIASES: u32 = 0x80044024;
const WBEMMOF_E_MULTIPLE_ALIASES_EN: &str =
    "(WBEMMOF_E_MULTIPLE_ALIASES) An alias cannot be used twice.";

const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2: u32 = 0x80044025;
const WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2_EN: &str =
    "(WBEMMOF_E_INCOMPATIBLE_FLAVOR_TYPES2) Combining Restricted, and ToInstance or ToSubClass is \
    not legal.";

const WBEMMOF_E_NO_ARRAYS_RETURNED: u32 = 0x80044026;
const WBEMMOF_E_NO_ARRAYS_RETURNED_EN: &str =
    "(WBEMMOF_E_NO_ARRAYS_RETURNED) Methods cannot return array values.";

const WBEMMOF_E_MUST_BE_IN_OR_OUT: u32 = 0x80044027;
const WBEMMOF_E_MUST_BE_IN_OR_OUT_EN: &str =
    "(WBEMMOF_E_MUST_BE_IN_OR_OUT) Arguments must have an In or Out qualifier.";

const WBEMMOF_E_INVALID_FLAGS_SYNTAX: u32 = 0x80044028;
const WBEMMOF_E_INVALID_FLAGS_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_FLAGS_SYNTAX) Flags syntax is not valid.";

const WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE: u32 = 0x80044029;
const WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE_EN: &str =
    "(WBEMMOF_E_EXPECTED_BRACE_OR_BAD_TYPE) The final brace and semi-colon for a class are missing.";

const WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE: u32 = 0x8004402A;
const WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE_EN: &str =
    "(WBEMMOF_E_UNSUPPORTED_CIMV22_QUAL_VALUE) A CIM version 2.2 feature is not supported for a \
    qualifier value.";

const WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE: u32 = 0x8004402B;
const WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE_EN: &str =
    "(WBEMMOF_E_UNSUPPORTED_CIMV22_DATA_TYPE) The CIM version 2.2 data type is not supported.";

const WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX: u32 = 0x8004402C;
const WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_DELETEINSTANCE_SYNTAX) The delete instance syntax is not valid. It should \
    be #pragma DeleteInstance(\"instancepath\", FAIL|NOFAIL)";

const WBEMMOF_E_INVALID_QUALIFIER_SYNTAX: u32 = 0x8004402D;
const WBEMMOF_E_INVALID_QUALIFIER_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_QUALIFIER_SYNTAX) The qualifier syntax is not valid. It should be \
    qualifiername:type=value,scope(class|instance), flavorname .";

const WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE: u32 = 0x8004402E;
const WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE_EN: &str =
    "(WBEMMOF_E_QUALIFIER_USED_OUTSIDE_SCOPE) The qualifier is used outside of its scope.";

const WBEMMOF_E_ERROR_CREATING_TEMP_FILE: u32 = 0x8004402F;
const WBEMMOF_E_ERROR_CREATING_TEMP_FILE_EN: &str =
    "(WBEMMOF_E_ERROR_CREATING_TEMP_FILE) Error creating temporary file. The temporary file is an \
    intermediate stage in the MOF compilation.";

const WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE: u32 = 0x80044030;
const WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE_EN: &str =
    "(WBEMMOF_E_ERROR_INVALID_INCLUDE_FILE) A file included in the MOF by the preprocessor command \
    #include is not valid.";

const WBEMMOF_E_INVALID_DELETECLASS_SYNTAX: u32 = 0x80044031;
const WBEMMOF_E_INVALID_DELETECLASS_SYNTAX_EN: &str =
    "(WBEMMOF_E_INVALID_DELETECLASS_SYNTAX) The syntax for the preprocessor commands #pragma \
    deleteinstance or #pragma deleteclass is not valid.";
