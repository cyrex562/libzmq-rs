
//! Compile-time settings for ZMQ.

/// Number of new messages in message pipe needed to trigger new memory
/// allocation. Setting this parameter to 256 decreases the impact of
/// memory allocation by approximately 99.6%
pub const MESSAGE_PIPE_GRANULARITY: usize = 256;

/// Commands in pipe per allocation event.
pub const COMMAND_PIPE_GRANULARITY: usize = 16;

/// Determines how often does socket poll for new commands when it
/// still has unprocessed messages to handle. Thus, if it is set to 100,
/// socket will process 100 inbound messages before doing the poll.
/// If there are no unprocessed messages available, poll is done
/// immediately. Decreasing the value trades overall latency for more
/// real-time behaviour (less latency peaks).
pub const INBOUND_POLL_RATE: usize = 100;

/// Maximal delta between high and low watermark.
pub const MAX_WM_DELTA: usize = 1024;

/// Maximum number of events the I/O thread can process in one go.
pub const MAX_IO_EVENTS: usize = 256;

/// Maximal batch size of packets forwarded by a ZMQ proxy.
/// Increasing this value improves throughput at the expense of
/// latency and fairness.
pub const PROXY_BURST_SIZE: usize = 1000;

/// Maximal delay to process command in API thread (in CPU ticks).
/// 3,000,000 ticks equals to 1 - 2 milliseconds on current CPUs.
/// Note that delay is only applied when there is continuous stream of
/// messages to process. If not so, commands are processed immediately.
pub const MAX_COMMAND_DELAY: usize = 3_000_000;

/// Low-precision clock precision in CPU ticks. 1ms. Value of 1000000
/// should be OK for CPU frequencies above 1GHz. If should work
/// reasonably well for CPU frequencies above 500MHz. For lower CPU
/// frequencies you may consider lowering this value to get best
/// possible latencies.
pub const CLOCK_PRECISION: usize = 1_000_000;

/// On some OSes the signaler has to be emulated using a TCP
/// connection. In such cases following port is used.
/// If 0, it lets the OS choose a free port without requiring use of a
/// global mutex. The original implementation of a Windows signaler
/// socket used port 5905 instead of letting the OS choose a free port.
pub const SIGNALER_PORT: u16 = 0;
