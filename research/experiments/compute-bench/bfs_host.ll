; Descriptor-only entry shared by sparse and pull graph experiments.
define void @wf_bench_bfs(ptr %edges, i64 %slots, i64 %pull, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %edges, 0
  %b = insertvalue { ptr, i64 } %a, i64 %slots, 1
  %r = call { ptr, i64 } @wf_bfs({ ptr, i64 } %b, i64 %pull)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_bfs_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %r = call i64 @wf_release_bfs({ ptr, i64 } %b)
  ret void
}
