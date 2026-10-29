# WM026 actual source check

Inspected baseline: `3586300ba1e5161c7f97a1ff5f797439966491d4`. Neither WM026 nor ZHWM026 occurs in the current executable catalog or rules registration; no WM026 implementation module exists. Thus this is not an already implemented transformation with only a damage-retention regression remaining. No new permission refusal or runtime failure occurred.

Both original images were viewed. WM026 image SHA256 `75a671983008abff26bfb01369d444eb7709a89ddab02cacc58da43fcd2ffdeb`; ZHWM026 image SHA256 `665aa2f70395be7ea99b8f8efbd8664ad79551fd5a0ea8fe0da2e52b619ec114`. The mist face actually says enemy hand count at most **three**, while the pinned transcription incorrectly said two. Only those two transcription occurrences are corrected here. This is source correction, not a new project ruling or executable implementation.

The expansion transcript `organized_content/rules/扩展说明书_四页_markdown整理版_v2.md:319` describes rotation and use of the other face's rules. The parent's damage/marker preservation direction is retained; it must not be implemented by calling the zone-reset helper, which clears per-instance state. Existing `remove_dead_with_snapshot` removes the physical card before dispatching Death; the generic lethal loop repeats while a lethal character remains. Simply keeping WM026 on the board while queueing its death trigger would therefore loop unless the serialized pending-death state is integrated with lethal discovery.

Remaining engineering: real front/form identity; death replacement and response continuation; preserved damage/markers and possible subsequent lethal mist death; preparation-step return with actual three-card boundary; normal entry and zone reset; Game/Room rejection/recovery; natural commands and applicable Native/WASM/UI/Worker validation. These are pending, not tested or completed. Whole-card count zero, public admission closed. Do not misrepresent this checkpoint as a damage fix, complete mechanism or independent review.

This checkpoint runs no game test suite and changes no executable rules. Existing review, Library HTTP 401 and push restrictions remain unchanged; no retries, push, merge or deployment.
