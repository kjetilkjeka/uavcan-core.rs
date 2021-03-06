# Uavcan.rs design goals

## Design Goals
- High robustness and safety
    - Allow error handling in an [idioatmic](https://doc.rust-lang.org/book/second-edition/ch09-00-error-handling.html) fashion
    - Following the rust API [Type safety guidelines](https://rust-lang-nursery.github.io/api-guidelines/type-safety.html)
    - Following the rust API [dependability guidelines](https://rust-lang-nursery.github.io/api-guidelines/dependability.html)
    - The tests should provide a reasonable coverage.
    - Be explicit in what kind of guarantees the library provides (and also in the lack of guarantees). 
- Feature-completeness, suitability for complex applications (like Libuavcan, unlike Libcanard). This includes:
    - rpc like features on top of service frames both from a caller and responder perspective. The caller should be able to choose between synchronous and asynchronous alternatives.
- Provide common application level functionality. To make the implementation as maintainable as possible, these should require as little knowledge of the internals as possibly. They will more often than not exist in their own crate depending on the other uavcan crates. Some of the feature that will be supported is listed here:
    - Network discovery
    - Time synchronization
    - Dynamic ID allocation server and client
- Ease of use
    - Follow the rust [API guidelines](https://rust-lang-nursery.github.io/api-guidelines/about.html)
    - Using "ease of use" as a metric when discussing addition/changes
- Assume as little as possible about the application and hardware
    - The library should be useful in the following scenarios:
        - Bare metal environment
        - RTOS/GPOS providing threads
        - low-level reactive schedulers such as real-time for the masses
    - Even in highly resource constrained environments the library should provide (at least) basic features
        - Only using the parts of the library that serialize/deserialize between Uavcan-frames and CAN-frames should be possible
    - Even though this flexibility will require some level of configuration it's just as important to maintain the ease of use.
        - The library should provide "sensible defaults" where applicable. Preferably, these defaults should be provided programmatically through good use of Rust features (such as the Default trait). Alternatively, they may be provided through documentation.
- Possible to extend for CAN-FD (after Uavcan supports this)
