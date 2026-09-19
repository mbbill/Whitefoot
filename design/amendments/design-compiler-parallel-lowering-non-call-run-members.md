Node: compiler/parallel-lowering

Decision: A permitted run's member that is not a call ends the overlap group the lowering builds from that run, so the group is the run's leading call-rooted prefix, because [PAR-1] now judges every adjacent statement pair and a run therefore carries members with no call at all, while the hand-out lowering has a form only for a call, and skipping such a member would claim an overlap for the pair that becomes adjacent only after the skip, which is a pair [PAR-1] never judged, instead of dropping non-call members from the group and keeping the calls on either side of them.

Rejected:
- Refusing to build any group from a run that contains a non-call member: rejected because the leading prefix of calls was judged pairwise like every other prefix, so refusing it would discard permission the checker granted for no reason the lowering can state.
