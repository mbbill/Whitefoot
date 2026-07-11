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
  %t26 = getelementptr i8, ptr %t3, i64 %t25
  %t27 = load i8, ptr %t26
  %t28 = alloca i8
  store i8 %t27, ptr %t28
  %t29 = load i64, ptr %t21
  %t30 = getelementptr i8, ptr %t3, i64 %t29
  %t31 = load i8, ptr %t30
  %t32 = alloca i8
  store i8 %t31, ptr %t32
  %t33 = load i64, ptr %t24
  %t34 = getelementptr i8, ptr %t3, i64 %t33
  %t35 = load i8, ptr %t34
  %t36 = alloca i8
  store i8 %t35, ptr %t36
  %t37 = load i8, ptr %t28
  %t38 = zext i8 %t37 to i32
  %t39 = alloca i32
  store i32 %t38, ptr %t39
  %t40 = load i8, ptr %t32
  %t41 = zext i8 %t40 to i32
  %t42 = alloca i32
  store i32 %t41, ptr %t42
  %t43 = load i8, ptr %t36
  %t44 = zext i8 %t43 to i32
  %t45 = alloca i32
  store i32 %t44, ptr %t45
  %t46 = load i32, ptr %t39
  %t47 = and i32 16, 31
  %t48 = shl i32 %t46, %t47
  %t49 = alloca i32
  store i32 %t48, ptr %t49
  %t50 = load i32, ptr %t42
  %t51 = and i32 8, 31
  %t52 = shl i32 %t50, %t51
  %t53 = alloca i32
  store i32 %t52, ptr %t53
  %t54 = load i32, ptr %t49
  %t55 = load i32, ptr %t53
  %t56 = or i32 %t54, %t55
  %t57 = alloca i32
  store i32 %t56, ptr %t57
  %t58 = load i32, ptr %t57
  %t59 = load i32, ptr %t45
  %t60 = or i32 %t58, %t59
  %t61 = alloca i32
  store i32 %t60, ptr %t61
  %t62 = load i32, ptr %t61
  %t63 = and i32 18, 31
  %t64 = lshr i32 %t62, %t63
  %t65 = alloca i32
  store i32 %t64, ptr %t65
  %t66 = load i32, ptr %t65
  %t67 = and i32 %t66, 63
  %t68 = alloca i32
  store i32 %t67, ptr %t68
  %t69 = load i32, ptr %t68
  %t70 = zext i32 %t69 to i64
  %t71 = alloca i64
  store i64 %t70, ptr %t71
  %t72 = load i32, ptr %t61
  %t73 = and i32 12, 31
  %t74 = lshr i32 %t72, %t73
  %t75 = alloca i32
  store i32 %t74, ptr %t75
  %t76 = load i32, ptr %t75
  %t77 = and i32 %t76, 63
  %t78 = alloca i32
  store i32 %t77, ptr %t78
  %t79 = load i32, ptr %t78
  %t80 = zext i32 %t79 to i64
  %t81 = alloca i64
  store i64 %t80, ptr %t81
  %t82 = load i32, ptr %t61
  %t83 = and i32 6, 31
  %t84 = lshr i32 %t82, %t83
  %t85 = alloca i32
  store i32 %t84, ptr %t85
  %t86 = load i32, ptr %t85
  %t87 = and i32 %t86, 63
  %t88 = alloca i32
  store i32 %t87, ptr %t88
  %t89 = load i32, ptr %t88
  %t90 = zext i32 %t89 to i64
  %t91 = alloca i64
  store i64 %t90, ptr %t91
  %t92 = load i32, ptr %t61
  %t93 = and i32 %t92, 63
  %t94 = alloca i32
  store i32 %t93, ptr %t94
  %t95 = load i32, ptr %t94
  %t96 = zext i32 %t95 to i64
  %t97 = alloca i64
  store i64 %t96, ptr %t97
  %t98 = load i64, ptr %t71
  %t99 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t98
  %t100 = load i8, ptr %t99
  %t101 = alloca i8
  store i8 %t100, ptr %t101
  %t102 = load i64, ptr %t81
  %t103 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t102
  %t104 = load i8, ptr %t103
  %t105 = alloca i8
  store i8 %t104, ptr %t105
  %t106 = load i64, ptr %t91
  %t107 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t106
  %t108 = load i8, ptr %t107
  %t109 = alloca i8
  store i8 %t108, ptr %t109
  %t110 = load i64, ptr %t97
  %t111 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t110
  %t112 = load i8, ptr %t111
  %t113 = alloca i8
  store i8 %t112, ptr %t113
  %t114 = load i64, ptr %t7
  %t115 = add i64 %t114, 1
  %t116 = alloca i64
  store i64 %t115, ptr %t116
  %t117 = load i64, ptr %t7
  %t118 = add i64 %t117, 2
  %t119 = alloca i64
  store i64 %t118, ptr %t119
  %t120 = load i64, ptr %t7
  %t121 = add i64 %t120, 3
  %t122 = alloca i64
  store i64 %t121, ptr %t122
  %t123 = load i8, ptr %t101
  %t124 = load i64, ptr %t7
  %t125 = getelementptr i8, ptr %t1, i64 %t124
  store i8 %t123, ptr %t125
  %t126 = load i8, ptr %t105
  %t127 = load i64, ptr %t116
  %t128 = getelementptr i8, ptr %t1, i64 %t127
  store i8 %t126, ptr %t128
  %t129 = load i8, ptr %t109
  %t130 = load i64, ptr %t119
  %t131 = getelementptr i8, ptr %t1, i64 %t130
  store i8 %t129, ptr %t131
  %t132 = load i8, ptr %t113
  %t133 = load i64, ptr %t122
  %t134 = getelementptr i8, ptr %t1, i64 %t133
  store i8 %t132, ptr %t134
  %t135 = load i64, ptr %t6
  %t136 = add i64 %t135, 3
  store i64 %t136, ptr %t6
  %t137 = load i64, ptr %t7
  %t138 = add i64 %t137, 4
  store i64 %t138, ptr %t7
  br label %L8
L9:
  %t139 = load i64, ptr %t5
  %t140 = load i64, ptr %t6
  %t141 = sub i64 %t139, %t140
  %t142 = alloca i64
  store i64 %t141, ptr %t142
  %t143 = load i64, ptr %t142
  %t144 = icmp eq i64 %t143, 1
  br i1 %t144, label %L146, label %L147
L146:
  %t148 = load i64, ptr %t6
  %t149 = getelementptr i8, ptr %t3, i64 %t148
  %t150 = load i8, ptr %t149
  %t151 = alloca i8
  store i8 %t150, ptr %t151
  %t152 = load i8, ptr %t151
  %t153 = zext i8 %t152 to i32
  %t154 = alloca i32
  store i32 %t153, ptr %t154
  %t155 = load i32, ptr %t154
  %t156 = and i32 2, 31
  %t157 = lshr i32 %t155, %t156
  %t158 = alloca i32
  store i32 %t157, ptr %t158
  %t159 = load i32, ptr %t158
  %t160 = and i32 %t159, 63
  %t161 = alloca i32
  store i32 %t160, ptr %t161
  %t162 = load i32, ptr %t161
  %t163 = zext i32 %t162 to i64
  %t164 = alloca i64
  store i64 %t163, ptr %t164
  %t165 = load i32, ptr %t154
  %t166 = and i32 4, 31
  %t167 = shl i32 %t165, %t166
  %t168 = alloca i32
  store i32 %t167, ptr %t168
  %t169 = load i32, ptr %t168
  %t170 = and i32 %t169, 63
  %t171 = alloca i32
  store i32 %t170, ptr %t171
  %t172 = load i32, ptr %t171
  %t173 = zext i32 %t172 to i64
  %t174 = alloca i64
  store i64 %t173, ptr %t174
  %t175 = load i64, ptr %t164
  %t176 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t175
  %t177 = load i8, ptr %t176
  %t178 = alloca i8
  store i8 %t177, ptr %t178
  %t179 = load i64, ptr %t174
  %t180 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t179
  %t181 = load i8, ptr %t180
  %t182 = alloca i8
  store i8 %t181, ptr %t182
  %t183 = load i64, ptr %t7
  %t184 = add i64 %t183, 1
  %t185 = alloca i64
  store i64 %t184, ptr %t185
  %t186 = load i64, ptr %t7
  %t187 = add i64 %t186, 2
  %t188 = alloca i64
  store i64 %t187, ptr %t188
  %t189 = load i64, ptr %t7
  %t190 = add i64 %t189, 3
  %t191 = alloca i64
  store i64 %t190, ptr %t191
  %t192 = load i8, ptr %t178
  %t193 = load i64, ptr %t7
  %t194 = getelementptr i8, ptr %t1, i64 %t193
  store i8 %t192, ptr %t194
  %t195 = load i8, ptr %t182
  %t196 = load i64, ptr %t185
  %t197 = getelementptr i8, ptr %t1, i64 %t196
  store i8 %t195, ptr %t197
  %t198 = load i64, ptr %t188
  %t199 = getelementptr i8, ptr %t1, i64 %t198
  store i8 61, ptr %t199
  %t200 = load i64, ptr %t191
  %t201 = getelementptr i8, ptr %t1, i64 %t200
  store i8 61, ptr %t201
  %t202 = load i64, ptr %t7
  %t203 = add i64 %t202, 4
  store i64 %t203, ptr %t7
  br label %L145
L147:
  br label %L145
L145:
  %t204 = load i64, ptr %t142
  %t205 = icmp eq i64 %t204, 2
  br i1 %t205, label %L207, label %L208
L207:
  %t209 = load i64, ptr %t6
  %t210 = add i64 %t209, 1
  %t211 = alloca i64
  store i64 %t210, ptr %t211
  %t212 = load i64, ptr %t6
  %t213 = getelementptr i8, ptr %t3, i64 %t212
  %t214 = load i8, ptr %t213
  %t215 = alloca i8
  store i8 %t214, ptr %t215
  %t216 = load i64, ptr %t211
  %t217 = getelementptr i8, ptr %t3, i64 %t216
  %t218 = load i8, ptr %t217
  %t219 = alloca i8
  store i8 %t218, ptr %t219
  %t220 = load i8, ptr %t215
  %t221 = zext i8 %t220 to i32
  %t222 = alloca i32
  store i32 %t221, ptr %t222
  %t223 = load i8, ptr %t219
  %t224 = zext i8 %t223 to i32
  %t225 = alloca i32
  store i32 %t224, ptr %t225
  %t226 = load i32, ptr %t222
  %t227 = and i32 8, 31
  %t228 = shl i32 %t226, %t227
  %t229 = alloca i32
  store i32 %t228, ptr %t229
  %t230 = load i32, ptr %t229
  %t231 = load i32, ptr %t225
  %t232 = or i32 %t230, %t231
  %t233 = alloca i32
  store i32 %t232, ptr %t233
  %t234 = load i32, ptr %t233
  %t235 = and i32 10, 31
  %t236 = lshr i32 %t234, %t235
  %t237 = alloca i32
  store i32 %t236, ptr %t237
  %t238 = load i32, ptr %t237
  %t239 = and i32 %t238, 63
  %t240 = alloca i32
  store i32 %t239, ptr %t240
  %t241 = load i32, ptr %t240
  %t242 = zext i32 %t241 to i64
  %t243 = alloca i64
  store i64 %t242, ptr %t243
  %t244 = load i32, ptr %t233
  %t245 = and i32 4, 31
  %t246 = lshr i32 %t244, %t245
  %t247 = alloca i32
  store i32 %t246, ptr %t247
  %t248 = load i32, ptr %t247
  %t249 = and i32 %t248, 63
  %t250 = alloca i32
  store i32 %t249, ptr %t250
  %t251 = load i32, ptr %t250
  %t252 = zext i32 %t251 to i64
  %t253 = alloca i64
  store i64 %t252, ptr %t253
  %t254 = load i32, ptr %t233
  %t255 = and i32 2, 31
  %t256 = shl i32 %t254, %t255
  %t257 = alloca i32
  store i32 %t256, ptr %t257
  %t258 = load i32, ptr %t257
  %t259 = and i32 %t258, 63
  %t260 = alloca i32
  store i32 %t259, ptr %t260
  %t261 = load i32, ptr %t260
  %t262 = zext i32 %t261 to i64
  %t263 = alloca i64
  store i64 %t262, ptr %t263
  %t264 = load i64, ptr %t243
  %t265 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t264
  %t266 = load i8, ptr %t265
  %t267 = alloca i8
  store i8 %t266, ptr %t267
  %t268 = load i64, ptr %t253
  %t269 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t268
  %t270 = load i8, ptr %t269
  %t271 = alloca i8
  store i8 %t270, ptr %t271
  %t272 = load i64, ptr %t263
  %t273 = getelementptr [64 x i8], ptr @__const_b64, i64 0, i64 %t272
  %t274 = load i8, ptr %t273
  %t275 = alloca i8
  store i8 %t274, ptr %t275
  %t276 = load i64, ptr %t7
  %t277 = add i64 %t276, 1
  %t278 = alloca i64
  store i64 %t277, ptr %t278
  %t279 = load i64, ptr %t7
  %t280 = add i64 %t279, 2
  %t281 = alloca i64
  store i64 %t280, ptr %t281
  %t282 = load i64, ptr %t7
  %t283 = add i64 %t282, 3
  %t284 = alloca i64
  store i64 %t283, ptr %t284
  %t285 = load i8, ptr %t267
  %t286 = load i64, ptr %t7
  %t287 = getelementptr i8, ptr %t1, i64 %t286
  store i8 %t285, ptr %t287
  %t288 = load i8, ptr %t271
  %t289 = load i64, ptr %t278
  %t290 = getelementptr i8, ptr %t1, i64 %t289
  store i8 %t288, ptr %t290
  %t291 = load i8, ptr %t275
  %t292 = load i64, ptr %t281
  %t293 = getelementptr i8, ptr %t1, i64 %t292
  store i8 %t291, ptr %t293
  %t294 = load i64, ptr %t284
  %t295 = getelementptr i8, ptr %t1, i64 %t294
  store i8 61, ptr %t295
  %t296 = load i64, ptr %t7
  %t297 = add i64 %t296, 4
  store i64 %t297, ptr %t7
  br label %L206
L208:
  br label %L206
L206:
  %t298 = load i64, ptr %t7
  ret i64 %t298
}
