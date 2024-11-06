#![allow(non_camel_case_types)]

/// Available poller implementation types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PollerType {
    KQUEUE,
    EPOLL,
    DEVPOLL,
    POLLSET,
    POLL,
    SELECT,
}

#[cfg(all(
    any(
        feature = "use-kqueue",
        feature = "use-epoll",
        feature = "use-devpoll",
        feature = "use-pollset",
        feature = "use-poll",
        feature = "use-select"
    ),
    not(any(
        all(feature = "use-kqueue", feature = "use-epoll"),
        all(feature = "use-kqueue", feature = "use-devpoll"),
        all(feature = "use-kqueue", feature = "use-pollset"),
        all(feature = "use-kqueue", feature = "use-poll"),
        all(feature = "use-kqueue", feature = "use-select"),
        all(feature = "use-epoll", feature = "use-devpoll"),
        all(feature = "use-epoll", feature = "use-pollset"),
        all(feature = "use-epoll", feature = "use-poll"),
        all(feature = "use-epoll", feature = "use-select"),
        all(feature = "use-devpoll", feature = "use-pollset"),
        all(feature = "use-devpoll", feature = "use-poll"),
        all(feature = "use-devpoll", feature = "use-select"),
        all(feature = "use-pollset", feature = "use-poll"),
        all(feature = "use-pollset", feature = "use-select"),
        all(feature = "use-poll", feature = "use-select"),
    ))
))]
pub fn get_poller_type() -> PollerType {
    #[cfg(feature = "use-kqueue")]
    return PollerType::KQUEUE;
    #[cfg(feature = "use-epoll")]
    return PollerType::EPOLL;
    #[cfg(feature = "use-devpoll")]
    return PollerType::DEVPOLL;
    #[cfg(feature = "use-pollset")]
    return PollerType::POLLSET;
    #[cfg(feature = "use-poll")]
    return PollerType::POLL;
    #[cfg(feature = "use-select")]
    return PollerType::SELECT;
    #[cfg(target_os = "gnu")]
    return PollerType::POLL;
    
    #[cfg(not(any(
        feature = "use-kqueue",
        feature = "use-epoll",
        feature = "use-devpoll",
        feature = "use-pollset",
        feature = "use-poll",
        feature = "use-select",
        target_os = "gnu"
    )))]
    compile_error!("No poller type selected");
}

// Note: The poll-based configuration would be handled through Cargo features
// rather than macros in Rust
