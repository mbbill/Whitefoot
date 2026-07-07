declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

define i32 @sign_of(i32 %x) {
entry:
  %t1 = icmp slt i32 %x, 0
  br i1 %t1, label %L3, label %L4
L3:
  ret i32 0
L4:
  %t5 = icmp eq i32 %x, 0
  br i1 %t5, label %L7, label %L8
L7:
  ret i32 1
L8:
  ret i32 2
}

define i32 @main() {
entry:
  %t1 = alloca i32
  store i32 40, ptr %t1
  %t2 = load i32, ptr %t1
  %t3 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t2, i32 2)
  %t4 = extractvalue {i32, i1} %t3, 0
  %t5 = extractvalue {i32, i1} %t3, 1
  br i1 %t5, label %L8, label %L7
L7:
  %t9 = icmp eq i32 %t4, 42
  br i1 %t9, label %L10, label %trap
L10:
  %t11 = call i32 @sign_of(i32 %t4)
  %t12 = icmp eq i32 %t11, 2
  br i1 %t12, label %L13, label %trap
L13:
  br label %L6
L8:
  ret i32 0
L6:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}
