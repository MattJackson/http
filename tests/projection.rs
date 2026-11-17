use bytes::Bytes;
use http::header::{HeaderMapPlan, HeaderName, HeaderValue, ProjectionError};
use http::{HeaderMap, Method};

#[test]
fn shared_names_and_long_methods_retain_original_backing() {
    let bytes = Bytes::from(vec![b'x'; 40]);
    let name = HeaderName::from_lowercase_shared(bytes.clone()).unwrap();
    let method = Method::from_maybe_shared(bytes.clone()).unwrap();
    assert_eq!(name.as_str().as_ptr(), bytes.as_ptr());
    assert_eq!(name.clone().as_str().as_ptr(), bytes.as_ptr());
    assert_eq!(method.as_str().as_ptr(), bytes.as_ptr());
    assert_eq!(method.clone().as_str().as_ptr(), bytes.as_ptr());
    assert_eq!(method, Method::from_bytes(&bytes).unwrap());
    assert_eq!(name, HeaderName::from_lowercase(&bytes).unwrap());
}

#[test]
fn shared_validation_preserves_existing_parser() {
    for input in [
        b"GET".as_slice(),
        b"x-custom",
        b"Content-Type",
        b"",
        b"bad name",
        b"bad\xff",
    ] {
        let bytes = Bytes::copy_from_slice(input);
        assert_eq!(
            Method::from_maybe_shared(bytes.clone()).ok(),
            Method::from_bytes(input).ok()
        );
        assert_eq!(
            HeaderName::from_lowercase_shared(bytes).ok(),
            HeaderName::from_lowercase(input).ok()
        );
    }
}

#[test]
fn planned_map_preserves_duplicate_order_and_checks_actual_count() {
    let plan = HeaderMapPlan::for_values(3).unwrap();
    assert_eq!(plan.values(), 3);
    assert!(plan.allocation_bytes() > 0);
    let fields = ["first", "second", "third"].into_iter().map(|value| {
        Ok((
            HeaderName::from_static("x-test"),
            HeaderValue::from_static(value),
        ))
    });
    let map = HeaderMap::try_from_plan(plan, fields).unwrap();
    assert_eq!(
        map.get_all("x-test")
            .iter()
            .map(|v| v.to_str().unwrap())
            .collect::<Vec<_>>(),
        ["first", "second", "third"]
    );
    assert!(
        HeaderMap::try_from_plan(HeaderMapPlan::for_values(1).unwrap(), std::iter::empty())
            .is_err()
    );
    assert!(HeaderMap::try_from_plan(
        HeaderMapPlan::for_values(0).unwrap(),
        std::iter::once(Ok((
            HeaderName::from_static("x"),
            HeaderValue::from_static("y")
        )))
    )
    .is_err());
    assert!(HeaderMap::try_from_plan(
        HeaderMapPlan::for_values(1).unwrap(),
        std::iter::once(Err(ProjectionError::InvalidField))
    )
    .is_err());
    assert!(HeaderMapPlan::for_values(usize::MAX).is_err());
    assert_eq!(HeaderMapPlan::for_values(0).unwrap().allocation_bytes(), 0);
}
