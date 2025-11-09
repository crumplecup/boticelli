// @generated automatically by Diesel CLI.

diesel::table! {
    model_responses (id) {
        id -> Uuid,
        created_at -> Timestamp,
        provider -> Varchar,
        model_name -> Varchar,
        request_messages -> Jsonb,
        request_temperature -> Nullable<Float4>,
        request_max_tokens -> Nullable<Int4>,
        request_model -> Nullable<Varchar>,
        response_outputs -> Jsonb,
        duration_ms -> Nullable<Int4>,
        error_message -> Nullable<Text>,
    }
}
