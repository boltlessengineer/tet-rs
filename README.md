# Tetries made with Rust

# TODOs

- [ ] make sure 7-bag system works well (discord live)
- [ ] make auto soft-drop (spawn new thread in loop)
  - [ ] lock after 15 motions(from first hit) or 0.5s timeout
- [ ] DAS/ARR support
  - send all key press/release event to main
  - if release event didn't occur less than `DAS_TIMEOUT`, run DAS motion (because it means keyboard is still pressed yet)
