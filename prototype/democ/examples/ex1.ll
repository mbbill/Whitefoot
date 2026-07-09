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
  %t2 = alloca i32
  %t3 = load i32, ptr %t1
  %t4 = call {i32, i1} @llvm.sadd.with.overflow.i32(i32 %t3, i32 2)
  %t5 = extractvalue {i32, i1} %t4, 0
  %t6 = extractvalue {i32, i1} %t4, 1
  %t10 = icmp eq i1 %t6, 0
  br i1 %t10, label %L8, label %L9
L8:
  store i32 %t5, ptr %t2
  br label %L7
L9:
  %t13 = icmp eq i1 %t6, 1
  br i1 %t13, label %L11, label %L12
L11:
  ret i32 0
L12:
  unreachable
L7:
  %t14 = load i32, ptr %t2
  %t15 = icmp eq i32 %t14, 42
  br i1 %t15, label %L16, label %trap
L16:
  %t17 = load i32, ptr %t2
  %t18 = call i32 @sign_of(i32 %t17)
  %t19 = alloca i32
  store i32 %t18, ptr %t19
  %t20 = load i32, ptr %t19
  %t24 = icmp eq i32 %t20, 0
  br i1 %t24, label %L22, label %L23
L22:
  ret i32 0
L23:
  %t27 = icmp eq i32 %t20, 1
  br i1 %t27, label %L25, label %L26
L25:
  ret i32 0
L26:
  %t30 = icmp eq i32 %t20, 2
  br i1 %t30, label %L28, label %L29
L28:
  %t31 = load i32, ptr %t2
  %t32 = icmp eq i32 %t31, 42
  br i1 %t32, label %L33, label %trap
L33:
  br label %L21
L29:
  unreachable
L21:
  ret i32 0
trap:
  call void @llvm.trap()
  unreachable
}
