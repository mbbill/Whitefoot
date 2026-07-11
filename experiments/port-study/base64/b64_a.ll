declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.ssub.with.overflow.i32(i32, i32)
declare {i32, i1} @llvm.smul.with.overflow.i32(i32, i32)
declare void @llvm.trap()

@__const_b64 = private unnamed_addr constant [64 x i8] [i8 65, i8 66, i8 67, i8 68, i8 69, i8 70, i8 71, i8 72, i8 73, i8 74, i8 75, i8 76, i8 77, i8 78, i8 79, i8 80, i8 81, i8 82, i8 83, i8 84, i8 85, i8 86, i8 87, i8 88, i8 89, i8 90, i8 97, i8 98, i8 99, i8 100, i8 101, i8 102, i8 103, i8 104, i8 105, i8 106, i8 107, i8 108, i8 109, i8 110, i8 111, i8 112, i8 113, i8 114, i8 115, i8 116, i8 117, i8 118, i8 119, i8 120, i8 121, i8 122, i8 48, i8 49, i8 50, i8 51, i8 52, i8 53, i8 54, i8 55, i8 56, i8 57, i8 43, i8 47]

define i64 @encode({ptr, i64} %out, {ptr, i64} %src) nounwind memory(argmem: readwrite, inaccessiblemem: write) {
entry:
  %t1 = extractvalue {ptr, i64} %out, 0
  %t2 = extractvalue {ptr, i64} %out, 1
  %t3 = extractvalue {ptr, i64} %src, 0
  %t4 = extractvalue {ptr, i64} %src, 1
  %t5 = alloca i64
  store i64 %t4, ptr %t5
  %t6 = alloca i64
  store i64 0, ptr %t6
  %t7 = alloca i64
  store i64 0, ptr %t7
  br label %L8
L8:
  %t10 = load i64, ptr %t5
  %t11 = load i64, ptr %t6
  %t12 = sub i64 %t10, %t11
  %t13 = alloca i64
  store i64 %t12, ptr %t13
  %t14 = load i64, ptr %t13
  %t15 = icmp ult i64 %t14, 3
  br i1 %t15, label %L17, label %L18
L17:
  br label %L9
L18:
  br label %L16
L16:
  %t19 = load i64, ptr %t6
  %t20 = add i64 %t19, 1
  %t21 = alloca i64
  store i64 %t20, ptr %t21
  %t22 = load i64, ptr %t6
  %t23 = add i64 %t22, 2
  %t24 = alloca i64
  store i64 %t23, ptr %t24
  %t25 = load i64, ptr %t6
  %t26 = icmp ult i64 %t25, %t4
  br i1 %t26, label %L27, label %trap
L27:
  %t28 = getelementptr i8, ptr %t3, i64 %t25
  %t29 = load i8, ptr %t28
  %t30 = alloca i8
  store i8 %t29, ptr %t30
  %t31 = load i64, ptr %t21
  %t32 = icmp ult i64 %t31, %t4
  br i1 %t32, label %L33, label %trap
L33:
  %t34 = getelementptr i8, ptr %t3, i64 %t31
  %t35 = load i8, ptr %t34
  %t36 = alloca i8
  store i8 %t35, ptr %t36
  %t37 = load i64, ptr %t24
  %t38 = icmp ult i64 %t37, %t4
  br i1 %t38, label %L39, label %trap
L39:
  %t40 = getelementptr i8, ptr %t3, i64 %t37
  %t41 = load i8, ptr %t40
  %t42 = alloca i8
  store i8 %t41, ptr %t42
  %t43 = load i8, ptr %t30
  %t44 = zext i8 %t43 to i32
  %t45 = alloca i32
  store i32 %t44, ptr %t45
  %t46 = load i8, ptr %t36
  %t47 = zext i8 %t46 to i32
  %t48 = alloca i32
  store i32 %t47, ptr %t48
  %t49 = load i8, ptr %t42
  %t50 = zext i8 %t49 to i32
  %t51 = alloca i32
  store i32 %t50, ptr %t51
  %t52 = load i32, ptr %t45
  %t53 = and i32 16, 31
  %t54 = shl i32 %t52, %t53
  %t55 = alloca i32
  store i32 %t54, ptr %t55
  %t56 = load i32, ptr %t48
  %t57 = and i32 8, 31
  %t58 = shl i32 %t56, %t57
  %t59 = alloca i32
  store i32 %t58, ptr %t59
  %t60 = load i32, ptr %t55
  %t61 = load i32, ptr %t59
  %t62 = or i32 %t60, %t61
  %t63 = alloca i32
  store i32 %t62, ptr %t63
  %t64 = load i32, ptr %t63
  %t65 = load i32, ptr %t51
  %t66 = or i32 %t64, %t65
  %t67 = alloca i32
  store i32 %t66, ptr %t67
  %t68 = load i32, ptr %t67
  %t69 = and i32 18, 31
  %t70 = lshr i32 %t68, %t69
  %t71 = alloca i32
  store i32 %t70, ptr %t71
  %t72 = load i32, ptr %t71
  %t73 = and i32 %t72, 63
  %t74 = alloca i32
  store i32 %t73, ptr %t74
  %t75 = load i32, ptr %t74
  %t76 = zext i32 %t75 to i64
  %t77 = alloca i64
  store i64 %t76, ptr %t77
  %t78 = load i32, ptr %t67
  %t79 = and i32 12, 31
  %t80 = lshr i32 %t78, %t79
  %t81 = alloca i32
  store i32 %t80, ptr %t81
  %t82 = load i32, ptr %t81
  %t83 = and i32 %t82, 63
  %t84 = alloca i32
  store i32 %t83, ptr %t84
  %t85 = load i32, ptr %t84
  %t86 = zext i32 %t85 to i64
  %t87 = alloca i64
  store i64 %t86, ptr %t87
  %t88 = load i32, ptr %t67
  %t89 = and i32 6, 31
  %t90 = lshr i32 %t88, %t89
  %t91 = alloca i32
  store i32 %t90, ptr %t91
  %t92 = load i32, ptr %t91
  %t93 = and i32 %t92, 63
  %t94 = alloca i32
  store i32 %t93, ptr %t94
  %t95 = load i32, ptr %t94
  %t96 = zext i32 %t95 to i64
  %t97 = alloca i64
  store i64 %t96, ptr %t97
  %t98 = load i32, ptr %t67
  %t99 = and i32 %t98, 63
  %t100 = alloca i32
  store i32 %t99, ptr %t100
  %t101 = load i32, ptr %t100
  %t102 = zext i32 %t101 to i64
  %t103 = alloca i64
  store i64 %t102, ptr %t103
  %t104 = load i64, ptr %t77
  %t105 = icmp ult i64 %t104, 64
  br i1 %t105, label %L106, label %trap
L106:
  %t107 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t104
  %t108 = load i8, ptr %t107
  %t109 = alloca i8
  store i8 %t108, ptr %t109
  %t110 = load i64, ptr %t87
  %t111 = icmp ult i64 %t110, 64
  br i1 %t111, label %L112, label %trap
L112:
  %t113 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t110
  %t114 = load i8, ptr %t113
  %t115 = alloca i8
  store i8 %t114, ptr %t115
  %t116 = load i64, ptr %t97
  %t117 = icmp ult i64 %t116, 64
  br i1 %t117, label %L118, label %trap
L118:
  %t119 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t116
  %t120 = load i8, ptr %t119
  %t121 = alloca i8
  store i8 %t120, ptr %t121
  %t122 = load i64, ptr %t103
  %t123 = icmp ult i64 %t122, 64
  br i1 %t123, label %L124, label %trap
L124:
  %t125 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t122
  %t126 = load i8, ptr %t125
  %t127 = alloca i8
  store i8 %t126, ptr %t127
  %t128 = load i64, ptr %t7
  %t129 = add i64 %t128, 1
  %t130 = alloca i64
  store i64 %t129, ptr %t130
  %t131 = load i64, ptr %t7
  %t132 = add i64 %t131, 2
  %t133 = alloca i64
  store i64 %t132, ptr %t133
  %t134 = load i64, ptr %t7
  %t135 = add i64 %t134, 3
  %t136 = alloca i64
  store i64 %t135, ptr %t136
  %t137 = load i8, ptr %t109
  %t138 = load i64, ptr %t7
  %t139 = icmp ult i64 %t138, %t2
  br i1 %t139, label %L140, label %trap
L140:
  %t141 = getelementptr i8, ptr %t1, i64 %t138
  store i8 %t137, ptr %t141
  %t142 = load i8, ptr %t115
  %t143 = load i64, ptr %t130
  %t144 = icmp ult i64 %t143, %t2
  br i1 %t144, label %L145, label %trap
L145:
  %t146 = getelementptr i8, ptr %t1, i64 %t143
  store i8 %t142, ptr %t146
  %t147 = load i8, ptr %t121
  %t148 = load i64, ptr %t133
  %t149 = icmp ult i64 %t148, %t2
  br i1 %t149, label %L150, label %trap
L150:
  %t151 = getelementptr i8, ptr %t1, i64 %t148
  store i8 %t147, ptr %t151
  %t152 = load i8, ptr %t127
  %t153 = load i64, ptr %t136
  %t154 = icmp ult i64 %t153, %t2
  br i1 %t154, label %L155, label %trap
L155:
  %t156 = getelementptr i8, ptr %t1, i64 %t153
  store i8 %t152, ptr %t156
  %t157 = load i64, ptr %t6
  %t158 = add i64 %t157, 3
  store i64 %t158, ptr %t6
  %t159 = load i64, ptr %t7
  %t160 = add i64 %t159, 4
  store i64 %t160, ptr %t7
  br label %L8
L9:
  %t161 = load i64, ptr %t5
  %t162 = load i64, ptr %t6
  %t163 = sub i64 %t161, %t162
  %t164 = alloca i64
  store i64 %t163, ptr %t164
  %t165 = load i64, ptr %t164
  %t166 = icmp eq i64 %t165, 1
  br i1 %t166, label %L168, label %L169
L168:
  %t170 = load i64, ptr %t6
  %t171 = icmp ult i64 %t170, %t4
  br i1 %t171, label %L172, label %trap
L172:
  %t173 = getelementptr i8, ptr %t3, i64 %t170
  %t174 = load i8, ptr %t173
  %t175 = alloca i8
  store i8 %t174, ptr %t175
  %t176 = load i8, ptr %t175
  %t177 = zext i8 %t176 to i32
  %t178 = alloca i32
  store i32 %t177, ptr %t178
  %t179 = load i32, ptr %t178
  %t180 = and i32 2, 31
  %t181 = lshr i32 %t179, %t180
  %t182 = alloca i32
  store i32 %t181, ptr %t182
  %t183 = load i32, ptr %t182
  %t184 = and i32 %t183, 63
  %t185 = alloca i32
  store i32 %t184, ptr %t185
  %t186 = load i32, ptr %t185
  %t187 = zext i32 %t186 to i64
  %t188 = alloca i64
  store i64 %t187, ptr %t188
  %t189 = load i32, ptr %t178
  %t190 = and i32 4, 31
  %t191 = shl i32 %t189, %t190
  %t192 = alloca i32
  store i32 %t191, ptr %t192
  %t193 = load i32, ptr %t192
  %t194 = and i32 %t193, 63
  %t195 = alloca i32
  store i32 %t194, ptr %t195
  %t196 = load i32, ptr %t195
  %t197 = zext i32 %t196 to i64
  %t198 = alloca i64
  store i64 %t197, ptr %t198
  %t199 = load i64, ptr %t188
  %t200 = icmp ult i64 %t199, 64
  br i1 %t200, label %L201, label %trap
L201:
  %t202 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t199
  %t203 = load i8, ptr %t202
  %t204 = alloca i8
  store i8 %t203, ptr %t204
  %t205 = load i64, ptr %t198
  %t206 = icmp ult i64 %t205, 64
  br i1 %t206, label %L207, label %trap
L207:
  %t208 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t205
  %t209 = load i8, ptr %t208
  %t210 = alloca i8
  store i8 %t209, ptr %t210
  %t211 = load i64, ptr %t7
  %t212 = add i64 %t211, 1
  %t213 = alloca i64
  store i64 %t212, ptr %t213
  %t214 = load i64, ptr %t7
  %t215 = add i64 %t214, 2
  %t216 = alloca i64
  store i64 %t215, ptr %t216
  %t217 = load i64, ptr %t7
  %t218 = add i64 %t217, 3
  %t219 = alloca i64
  store i64 %t218, ptr %t219
  %t220 = load i8, ptr %t204
  %t221 = load i64, ptr %t7
  %t222 = icmp ult i64 %t221, %t2
  br i1 %t222, label %L223, label %trap
L223:
  %t224 = getelementptr i8, ptr %t1, i64 %t221
  store i8 %t220, ptr %t224
  %t225 = load i8, ptr %t210
  %t226 = load i64, ptr %t213
  %t227 = icmp ult i64 %t226, %t2
  br i1 %t227, label %L228, label %trap
L228:
  %t229 = getelementptr i8, ptr %t1, i64 %t226
  store i8 %t225, ptr %t229
  %t230 = load i64, ptr %t216
  %t231 = icmp ult i64 %t230, %t2
  br i1 %t231, label %L232, label %trap
L232:
  %t233 = getelementptr i8, ptr %t1, i64 %t230
  store i8 61, ptr %t233
  %t234 = load i64, ptr %t219
  %t235 = icmp ult i64 %t234, %t2
  br i1 %t235, label %L236, label %trap
L236:
  %t237 = getelementptr i8, ptr %t1, i64 %t234
  store i8 61, ptr %t237
  %t238 = load i64, ptr %t7
  %t239 = add i64 %t238, 4
  store i64 %t239, ptr %t7
  br label %L167
L169:
  br label %L167
L167:
  %t240 = load i64, ptr %t164
  %t241 = icmp eq i64 %t240, 2
  br i1 %t241, label %L243, label %L244
L243:
  %t245 = load i64, ptr %t6
  %t246 = add i64 %t245, 1
  %t247 = alloca i64
  store i64 %t246, ptr %t247
  %t248 = load i64, ptr %t6
  %t249 = icmp ult i64 %t248, %t4
  br i1 %t249, label %L250, label %trap
L250:
  %t251 = getelementptr i8, ptr %t3, i64 %t248
  %t252 = load i8, ptr %t251
  %t253 = alloca i8
  store i8 %t252, ptr %t253
  %t254 = load i64, ptr %t247
  %t255 = icmp ult i64 %t254, %t4
  br i1 %t255, label %L256, label %trap
L256:
  %t257 = getelementptr i8, ptr %t3, i64 %t254
  %t258 = load i8, ptr %t257
  %t259 = alloca i8
  store i8 %t258, ptr %t259
  %t260 = load i8, ptr %t253
  %t261 = zext i8 %t260 to i32
  %t262 = alloca i32
  store i32 %t261, ptr %t262
  %t263 = load i8, ptr %t259
  %t264 = zext i8 %t263 to i32
  %t265 = alloca i32
  store i32 %t264, ptr %t265
  %t266 = load i32, ptr %t262
  %t267 = and i32 8, 31
  %t268 = shl i32 %t266, %t267
  %t269 = alloca i32
  store i32 %t268, ptr %t269
  %t270 = load i32, ptr %t269
  %t271 = load i32, ptr %t265
  %t272 = or i32 %t270, %t271
  %t273 = alloca i32
  store i32 %t272, ptr %t273
  %t274 = load i32, ptr %t273
  %t275 = and i32 10, 31
  %t276 = lshr i32 %t274, %t275
  %t277 = alloca i32
  store i32 %t276, ptr %t277
  %t278 = load i32, ptr %t277
  %t279 = and i32 %t278, 63
  %t280 = alloca i32
  store i32 %t279, ptr %t280
  %t281 = load i32, ptr %t280
  %t282 = zext i32 %t281 to i64
  %t283 = alloca i64
  store i64 %t282, ptr %t283
  %t284 = load i32, ptr %t273
  %t285 = and i32 4, 31
  %t286 = lshr i32 %t284, %t285
  %t287 = alloca i32
  store i32 %t286, ptr %t287
  %t288 = load i32, ptr %t287
  %t289 = and i32 %t288, 63
  %t290 = alloca i32
  store i32 %t289, ptr %t290
  %t291 = load i32, ptr %t290
  %t292 = zext i32 %t291 to i64
  %t293 = alloca i64
  store i64 %t292, ptr %t293
  %t294 = load i32, ptr %t273
  %t295 = and i32 2, 31
  %t296 = shl i32 %t294, %t295
  %t297 = alloca i32
  store i32 %t296, ptr %t297
  %t298 = load i32, ptr %t297
  %t299 = and i32 %t298, 63
  %t300 = alloca i32
  store i32 %t299, ptr %t300
  %t301 = load i32, ptr %t300
  %t302 = zext i32 %t301 to i64
  %t303 = alloca i64
  store i64 %t302, ptr %t303
  %t304 = load i64, ptr %t283
  %t305 = icmp ult i64 %t304, 64
  br i1 %t305, label %L306, label %trap
L306:
  %t307 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t304
  %t308 = load i8, ptr %t307
  %t309 = alloca i8
  store i8 %t308, ptr %t309
  %t310 = load i64, ptr %t293
  %t311 = icmp ult i64 %t310, 64
  br i1 %t311, label %L312, label %trap
L312:
  %t313 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t310
  %t314 = load i8, ptr %t313
  %t315 = alloca i8
  store i8 %t314, ptr %t315
  %t316 = load i64, ptr %t303
  %t317 = icmp ult i64 %t316, 64
  br i1 %t317, label %L318, label %trap
L318:
  %t319 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t316
  %t320 = load i8, ptr %t319
  %t321 = alloca i8
  store i8 %t320, ptr %t321
  %t322 = load i64, ptr %t7
  %t323 = add i64 %t322, 1
  %t324 = alloca i64
  store i64 %t323, ptr %t324
  %t325 = load i64, ptr %t7
  %t326 = add i64 %t325, 2
  %t327 = alloca i64
  store i64 %t326, ptr %t327
  %t328 = load i64, ptr %t7
  %t329 = add i64 %t328, 3
  %t330 = alloca i64
  store i64 %t329, ptr %t330
  %t331 = load i8, ptr %t309
  %t332 = load i64, ptr %t7
  %t333 = icmp ult i64 %t332, %t2
  br i1 %t333, label %L334, label %trap
L334:
  %t335 = getelementptr i8, ptr %t1, i64 %t332
  store i8 %t331, ptr %t335
  %t336 = load i8, ptr %t315
  %t337 = load i64, ptr %t324
  %t338 = icmp ult i64 %t337, %t2
  br i1 %t338, label %L339, label %trap
L339:
  %t340 = getelementptr i8, ptr %t1, i64 %t337
  store i8 %t336, ptr %t340
  %t341 = load i8, ptr %t321
  %t342 = load i64, ptr %t327
  %t343 = icmp ult i64 %t342, %t2
  br i1 %t343, label %L344, label %trap
L344:
  %t345 = getelementptr i8, ptr %t1, i64 %t342
  store i8 %t341, ptr %t345
  %t346 = load i64, ptr %t330
  %t347 = icmp ult i64 %t346, %t2
  br i1 %t347, label %L348, label %trap
L348:
  %t349 = getelementptr i8, ptr %t1, i64 %t346
  store i8 61, ptr %t349
  %t350 = load i64, ptr %t7
  %t351 = add i64 %t350, 4
  store i64 %t351, ptr %t7
  br label %L242
L244:
  br label %L242
L242:
  %t352 = load i64, ptr %t7
  ret i64 %t352
trap:
  call void @llvm.trap()
  unreachable
}
