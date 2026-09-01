declare i64 @bundle_target(i64)
declare i64 @metadata_target(i64)

define i64 @unsupported_forms_caller(i64 %x) {
entry:
  %bundled = call i64 @bundle_target(i64 %x) #0 [ "deopt"(i32 7) ]
  %metadata = call i64 @metadata_target(i64 %bundled), !nounwind !0, !willreturn !0, !readnone !0
  call void asm sideeffect "// quoted @bundle_target must not resolve", ""()
  ret i64 %metadata
}

attributes #0 = { nounwind willreturn memory(none) }
!0 = !{i32 1}
