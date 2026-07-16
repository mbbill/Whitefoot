"""Hand-transcribed AST for the 48 M1 programs (../m1extract/programs.json).

No parser: each program is built with the constructors from checker.py. Where a
pseudo-code snippet is ambiguous, the reading chosen is the one matching the
stated 'why', noted in a comment. Each entry is (id, Program, expected, canonical).
"""

from checker import (
    Place, TypeDecl, Field, Variant, TypeInfo, Sig, ParamSpec,
    ArgOwn, ArgBorrow, ArgValue,
    SLetNew, SLetCall, SLetConstruct, SLetMove, SRebind, SBlock,
    SMatch, MatchArm, SLoop, SBreak, SReturn, SPar,
    Func, Program,
)

# short helpers -------------------------------------------------------------
def _pl(p):   return p if isinstance(p, Place) else Place(p)
def B(m, p):  return ArgBorrow(m, _pl(p))
def O(p):     return ArgOwn(_pl(p))
V = ArgValue
def P(root, *path): return Place(root, tuple(path))

def cont(name, mode, region, ty):
    return ParamSpec(name, mode, region=region, confined=False, type_name=ty)
def tok(name, mode, ty, kind, regs):
    return ParamSpec(name, mode, confined=True, type_name=ty, kind=kind, region_args=regs)
def val(name):
    return ParamSpec(name, 'own')

def main(body):
    return Func(Sig('main'), body, is_main=True)

PROGRAMS = []
def add(pid, prog, expected, canonical=False):
    PROGRAMS.append((pid, prog, expected, canonical))


# ==========================================================================
# Canonical P1-P6
# ==========================================================================

# P1 — sequential unique pushes: no token outlives its statement.
add("P1", Program("P1", funcs=[main([
    SLetNew('s', 'seq'),
    SLetCall(None, 'push', [B('&uniq', 's'), V()]),
    SLetCall(None, 'push', [B('&uniq', 's'), V()]),
])]), "ACCEPT", True)

# P2 — insert while a uniq Entry token is live on t: R6 freeze rejects insert.
add("P2", Program("P2", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
    SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),   # <- rejects (R6)
    SLetCall(None, 'fill', [O('e'), V()]),
])]), "REJECT", True)

# P3 — mutate t through remove while a shr Cursor is live: uniq mint rejects.
add("P3", Program("P3", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLoop([
        SLetCall('n', 'next', [B('&uniq', 'c')]),
        SMatch(Place('n'), None, [                      # value match on option
            MatchArm('Some', 'kv', False, [
                SLetCall(None, 'remove', [B('&uniq', 't'), V()]),  # <- rejects (R6)
            ]),
            MatchArm('None', None, False, [SBreak()]),
        ]),
    ]),
])]), "REJECT", True)

# P4 — condvar wait loop with rebind: reissue keeps the loan balanced.
add("P4", Program("P4", funcs=[
    Func(Sig('worker', ('a',),
             [cont('m', '&', 'a', 'mutex'), cont('cv', '&', 'a', 'condvar')]),
         [
            SLetCall('g', 'lock', [B('&', 'm')]),
            SLoop([
                SLetCall('p', 'predicate', [B('&', 'g')]),
                SMatch(Place('p'), None, [
                    MatchArm('Done', None, False, [SBreak()]),
                    MatchArm('Wait', None, False, [
                        SLetCall('g2', 'wait', [B('&', 'cv'), B('&', 'm'), O('g')]),
                        SRebind('g', 'g2'),
                    ]),
                ]),
            ]),
         ]),
]), "ACCEPT", True)

# P5 — feed(&b, &uniq pa): pa's brand is a, tied place is b -> R10(a) rejects.
add("P5", Program("P5",
    funcs=[
        Func(Sig('feed', ('r',),
                 [cont('r', '&', 'r', 'ring'),
                  tok('p', '&uniq', 'Prod', 'shr', ('r',)), val('v')]),
             None),                                    # body "..." — trusted
        main([
            SLetNew('a', 'ring'),
            SLetNew('b', 'ring'),
            SLetCall('pa', 'producer', [B('&', 'a')]),
            SLetCall('cb', 'consumer', [B('&', 'b')]),
            SLetCall(None, 'feed', [B('&', 'b'), B('&uniq', 'pa'), V()]),  # <- rejects
        ]),
    ]), "REJECT", True)

# P6 — split-unique data + replicate-shared Env over a table: legal par.
add("P6", Program("P6",
    types=[TypeDecl('Env', True, 'shr', ('e',),
                    fields=[Field('lut', borrow_mode='&', borrow_region='e',
                                  region_args=('e',))])],
    funcs=[
        Func(Sig('body', ('c', 'e'),
                 [cont('chunk', '&uniq', 'c', 'chunkview'),
                  tok('env', '&', 'Env', 'shr', ('e',))]),
             None),                                    # body "..." — trusted
        Func(Sig('process', ('a',),
                 [cont('data', '&uniq', 'a', 'buffer'),
                  cont('tbl', '&', 'a', 'table')]),
             [
                SLetConstruct('env', 'Env', None, [('lut', B('&', 'tbl'))]),
                SPar('for_chunks',
                     [('split', B('&uniq', 'data')), ('replicate', B('&', 'env'))],
                     'body'),
                SLetCall(None, 'set', [B('&uniq', 'data'), V(), V()]),
             ]),
    ]), "ACCEPT", True)


# ==========================================================================
# Corpus (42) — each keyed to its programs.json 'why'.
# ==========================================================================

# A1 — R9/R13: leak() returns a token whose region is a local block region,
# not a region parameter of leak; declared with an untied result region.
add("A1/C1-escape-by-return", Program("A1", funcs=[
    Func(Sig('leak', ('x',), [], TypeInfo('Entry', True, 'uniq', ('x',))), None),
]), "REJECT")

# A2 — R1: a non-confined struct declares a confined (Entry) field.
add("A2/C2-token-in-struct-or-heap", Program("A2", types=[
    TypeDecl('Holder', False, fields=[
        Field('e', confined=True, kind='uniq', type_name='Entry', region_args=('h',))]),
]), "REJECT")

# A3 — R5/R6: a second uniq issue while (t,uniq,e1) is live. ('let t' implied.)
add("A3/C3-two-unique-tokens", Program("A3", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('e1', 'entry', [B('&uniq', 't'), V()]),
    SLetCall('e2', 'entry', [B('&uniq', 't'), V()]),        # <- rejects
])]), "REJECT")

# A4 — shr regime: two cursors + a &-receiver get coexist. ('let t' implied.)
add("A4/C5-two-shared-cursors", Program("A4", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c1', 'range', [B('&', 't'), V(), V()]),
    SLetCall('c2', 'range', [B('&', 't'), V(), V()]),
    SLetCall('v', 'get', [B('&', 't'), V()]),
    SLetCall('n1', 'next', [B('&uniq', 'c1')]),
    SLetCall('n2', 'next', [B('&uniq', 'c2')]),
])]), "ACCEPT")

# A5 — R5/R6: uniq issue while a shr entry lives on t (no upgrade).
add("A5/C4-shared-then-unique-upgrade", Program("A5", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),         # <- rejects
])]), "REJECT")

# A6 — R6: moving t while a live entry overlaps it.
add("A6/C6-move-while-loaned", Program("A6", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetMove('t2', 't'),                                    # <- rejects
])]), "REJECT")

# A7 — R12: fall-through arms disagree on loans(m) and g's liveness.
add("A7/C7-arm-imbalanced-consume", Program("A7", funcs=[
    Func(Sig('f7', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SMatch(Place('flag'), None, [
            MatchArm('T', None, False, [SLetCall(None, 'unlock_via', [B('&', 'm'), O('g')])]),
            MatchArm('F', None, False, [SLetCall(None, 'noop', [])]),
        ]),
    ]),
]), "REJECT")

# A8 — wrapped-writer bless: helper issues, returns; caller fills then inserts.
add("A8/C8-helper-issue-and-reuse", Program("A8", funcs=[
    Func(Sig('reserve', ('t',),
             [cont('t', '&uniq', 't', 'table'), val('k')],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SLetCall('e', 'entry', [B('&uniq', 't'), V()]), SReturn('e')]),
    main([
        SLetNew('tt', 'table'),
        SLetCall('e', 'reserve', [B('&uniq', 'tt'), V()]),
        SLetCall(None, 'fill', [O('e'), V()]),
        SLetCall(None, 'insert', [B('&uniq', 'tt'), V(), V()]),
    ]),
]), "ACCEPT")

# A9 — R12: back edge (e dead, loans empty) differs from entry (e live).
add("A9-preloop-token-consumed-in-body", Program("A9", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
    SLoop([
        SLetCall(None, 'fill', [O('e'), V()]),
        SMatch(Place('done'), None, [
            MatchArm('T', None, False, [SBreak()]),
            MatchArm('F', None, False, [SLetCall(None, 'noop', [])]),
        ]),
    ]),
])]), "REJECT")

# A10 — per-iteration token: issue/consume balance; back edge equals entry.
add("A10-per-iteration-token", Program("A10", funcs=[main([
    SLetNew('t', 'table'),
    SLoop([
        SLetCall('k', 'next_k', []),
        SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
        SLetCall(None, 'fill', [O('e'), V()]),
        SMatch(Place('done'), None, [
            MatchArm('T', None, False, [SBreak()]),
            MatchArm('F', None, False, [SLetCall(None, 'noop', [])]),
        ]),
    ]),
    SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
])]), "ACCEPT")

# A11 — honest composition: seed (x,shr,p); retire consumes; y.producer issues.
add("A11-cross-source-reissue", Program("A11", funcs=[
    Func(Sig('transfer', ('a', 'b'),
             [cont('x', '&', 'a', 'ring'), cont('y', '&', 'b', 'ring'),
              tok('p', 'own', 'Prod', 'shr', ('a',))],
             TypeInfo('Prod', True, 'shr', ('b',))),
         [
            SLetCall(None, 'retire', [B('&', 'x'), O('p')]),
            SLetCall('q', 'producer', [B('&', 'y')]),
            SReturn('q'),
         ]),
    main([
        SLetNew('a', 'ring'),
        SLetNew('b', 'ring'),
        SLetCall('pa', 'producer', [B('&', 'a')]),
        SLetCall('pb', 'transfer', [B('&', 'a'), B('&', 'b'), O('pa')]),
        SLetCall('r', 'endpoint_reset', [B('&', 'a')]),
    ]),
]), "ACCEPT")

# A12 — R9: result region 't ties to two borrow-mode parameters.
add("A12-ambiguous-region-signature", Program("A12", funcs=[
    Func(Sig('pick', ('t',),
             [cont('x', '&', 't', 'table'), cont('y', '&', 't', 'table'), val('sel')],
             TypeInfo('Cursor', True, 'shr', ('t',))), None),
]), "REJECT")

# C9 — shr regime read-under-cursor; back edge equals entry.
add("C9-loop-carried-cursor-reads", Program("C9", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLoop([
        SLetCall('n', 'next', [B('&uniq', 'c')]),
        SMatch(Place('n'), None, [
            MatchArm('Some', 'x', False, [SLetCall('v', 'get', [B('&', 't'), V()])]),
            MatchArm('None', None, False, [SBreak()]),
        ]),
    ]),
    SLetCall('w', 'get', [B('&', 't'), V()]),
])]), "ACCEPT")

# C10 — R12: Wait arm ends with g dead (g2 auto-consumed), disagreeing w/ entry.
add("C10-loop-reissue-without-rebind", Program("C10", funcs=[
    Func(Sig('f10', ('a',),
             [cont('m', '&', 'a', 'mutex'), cont('cv', '&', 'a', 'condvar')]),
         [
            SLetCall('g', 'lock', [B('&', 'm')]),
            SLoop([
                SLetCall('pv', 'predicate', [B('&', 'g')]),
                SMatch(Place('pv'), None, [
                    MatchArm('Done', None, False, [SBreak()]),
                    MatchArm('Wait', None, False, [
                        SLetCall('g2', 'wait', [B('&', 'cv'), B('&', 'm'), O('g')]),
                    ]),
                ]),
            ]),
         ]),
]), "REJECT")

# C11 — sequential uniq sessions (incl. inner-scope auto-consume) then shr.
add("C11-early-consume-then-reopen", Program("C11", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
    SLetCall(None, 'fill', [O('e'), V()]),
    SLetCall('e2', 'entry', [B('&uniq', 't'), V()]),
    SLetCall(None, 'fill', [O('e2'), V()]),
    SBlock([SLetCall('e3', 'entry', [B('&uniq', 't'), V()])]),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('x', 'get', [B('&', 't'), V()]),
])]), "ACCEPT")

# C12 — R8 renames e->w at construction; totality forces threading t; consume.
add("C12-wrap-then-helper-consume", Program("C12",
    types=[TypeDecl('Wrap', True, 'uniq', ('r',), fields=[
        Field('inner', confined=True, kind='uniq', type_name='Entry', region_args=('r',))])],
    funcs=[
        Func(Sig('finish', ('r',),
                 [cont('t', '&', 'r', 'table'), tok('w', 'own', 'Wrap', 'uniq', ('r',))]),
             None),
        main([
            SLetNew('t', 'table'),
            SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
            SLetConstruct('w', 'Wrap', None, [('inner', O('e'))]),
            SLetCall(None, 'finish', [B('&', 't'), O('w')]),
            SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
        ]),
    ]), "ACCEPT")

# C13 — R10(a): g1's source place m1 differs from the tied argument place m2.
add("C13-relock-brand-confusion", Program("C13", funcs=[
    Func(Sig('relock', ('r',),
             [cont('m', '&', 'r', 'mutex'), tok('g', 'own', 'Guard', 'uniq', ('r',))],
             TypeInfo('Guard', True, 'uniq', ('r',))), None),
    Func(Sig('f13', ('a',),
             [cont('m1', '&', 'a', 'mutex'), cont('m2', '&', 'a', 'mutex')]),
         [
            SLetCall('g1', 'lock', [B('&', 'm1')]),
            SLetCall('gx', 'relock', [B('&', 'm2'), O('g1')]),   # <- rejects
         ]),
]), "REJECT")

# C14 — R14: a holder holding a uniq entry cannot pass a replicate-shared slot.
add("C14-uniq-token-into-replicate-slot", Program("C14", funcs=[
    Func(Sig('f14', ('a',),
             [cont('m', '&', 'a', 'mutex'), cont('data', '&uniq', 'a', 'buffer')]),
         [
            SLetCall('g', 'lock', [B('&', 'm')]),
            SPar('for_chunks',
                 [('split', B('&uniq', 'data')), ('replicate', B('&', 'g'))], 'body'),
         ]),
]), "REJECT")

# ATK-gut — R9: 'u ties to zero borrow-mode parameters (undeclarable).
add("ATK-gut-untied-param", Program("ATKgut", funcs=[
    Func(Sig('gut', ('t', 'u'),
             [cont('t', '&uniq', 't', 'table'), tok('e', 'own', 'Entry', 'uniq', ('u',))]),
         [SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
          SLetCall(None, 'fill', [O('e'), V()])]),
]), "REJECT")

# ATK-launder-strength — R5: e2 reissued uniq, so t.range's shr issue rejects.
add("ATK-launder-strength", Program("ATKlaunder", funcs=[
    Func(Sig('launder', ('t',),
             [cont('t', '&', 't', 'table'), tok('e', 'own', 'Entry', 'uniq', ('t',))],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SReturn('e')]),
    main([
        SLetNew('t', 'table'),
        SLetCall('e', 'entry', [B('&uniq', 't'), V()]),        # implied source token
        SLetCall('e2', 'launder', [B('&', 't'), O('e')]),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),       # <- rejects
        SLetCall('n1', 'next', [B('&uniq', 'c')]),
        SLetCall(None, 'fill', [O('e2'), V()]),
    ]),
]), "REJECT")

# ATK-probe — R5/R6: the R10(c) seed keeps the caller's uniq loan; range rejects.
add("ATK-probe-callee-blind", Program("ATKprobe", funcs=[
    Func(Sig('probe', ('t',),
             [cont('t', '&', 't', 'table'), tok('e', 'own', 'Entry', 'uniq', ('t',))]),
         [
            SLetCall('c', 'range', [B('&', 't'), V(), V()]),   # <- rejects under seed
            SLetCall('n', 'next', [B('&uniq', 'c')]),
            SLetCall(None, 'fill', [O('e'), V()]),
            SLetCall('n2', 'next', [B('&uniq', 'c')]),
         ]),
]), "REJECT")

# ATK-refresh — R6: insert under the seeded uniq entry (statement doesn't consume e).
add("ATK-refresh-reissue-laundering", Program("ATKrefresh", funcs=[
    Func(Sig('refresh', ('t',),
             [cont('t', '&uniq', 't', 'table'), tok('e', 'own', 'Entry', 'uniq', ('t',))],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),   # <- rejects
          SReturn('e')]),
]), "REJECT")

# ATK-relocate — R5/R6: a uniq issue while the seeded (t,uniq,e) is live.
add("ATK-relocate-two-uniq", Program("ATKrelocate", funcs=[
    Func(Sig('relocate', ('t',),
             [cont('t', '&uniq', 't', 'table'), tok('e', 'own', 'Entry', 'uniq', ('t',))],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SLetCall('e2', 'entry', [B('&uniq', 't'), V()]),         # <- rejects
          SReturn('e2')]),
]), "REJECT")

# ATK-close-one — R6: consume-exception needs the consumed holder's entry SOLE.
add("ATK-close-one-sibling", Program("ATKcloseone", funcs=[main([
    SLetNew('tt', 'table'),
    SLetCall('c1', 'range', [B('&', 'tt'), V(), V()]),
    SLetCall('c2', 'range', [B('&', 'tt'), V(), V()]),
    SLetCall(None, 'close_one', [B('&uniq', 'tt'), O('c1')]),      # <- rejects
    SLetCall('n', 'next', [B('&uniq', 'c2')]),
])]), "REJECT")

# ATK-delete-at — R6 sole-loan exception: surrender the only token while mutating.
add("ATK-delete-at-sole-cursor", Program("ATKdeleteat", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall(None, 'delete_at', [B('&uniq', 't'), O('c')]),
])]), "ACCEPT")

# ATK-maybetok — R5: construction leaves a region parameter unbound (no capture).
add("ATK-maybetok-none-forgery", Program("ATKmaybe",
    types=[TypeDecl('MaybeTok', True, 'uniq', ('r',), is_enum=True, variants=[
        Variant('Some', [Field('f', confined=True, kind='uniq',
                               type_name='Entry', region_args=('r',))]),
        Variant('None', [])])],
    funcs=[main([
        SLetNew('tt', 'table'),
        SLetConstruct('none', 'MaybeTok', 'None', []),            # <- rejects
        SLetCall('e2', 'open', [B('&uniq', 'tt'), O('none')]),
    ])]), "REJECT")

# ATK-wview — R1 coherence: confined(shr) transitively holds a &uniq field.
add("ATK-wview-kind-incoherence", Program("ATKwview", types=[
    TypeDecl('WView', True, 'shr', ('r',), fields=[
        Field('p', borrow_mode='&uniq', borrow_region='r', region_args=('r',))]),
]), "REJECT")

# ATK-cloak — R1 coherence: shr wrapper containing a confined(uniq) field.
add("ATK-cloak-kind", Program("ATKcloak", types=[
    TypeDecl('Cloak', True, 'shr', ('r',), fields=[
        Field('inner', confined=True, kind='uniq', type_name='Entry', region_args=('r',))]),
]), "REJECT")

# ATK-db — R4/R6: entry freezes db.t (and db), not db.u; insert-before-fill rejects.
add("ATK-db-struct-field-place", Program("ATKdb",
    types=[TypeDecl('Db', False, fields=[
        Field('t', type_name='table'), Field('u', type_name='table')])],
    funcs=[main([
        SLetNew('db', 'Db'),
        SLetCall('e', 'entry', [B('&uniq', P('db', 't')), V()]),
        SLetCall(None, 'insert', [B('&uniq', P('db', 't')), V(), V()]),   # <- rejects
        SLetCall(None, 'fill', [O('e'), V()]),
    ])]), "REJECT")

# ATK-two-struct-rings — R10(a): pt's source two.a differs from tied place two.b.
add("ATK-two-struct-rings", Program("ATKtworings",
    types=[TypeDecl('Two', False, fields=[
        Field('a', type_name='ring'), Field('b', type_name='ring')])],
    funcs=[
        Func(Sig('feed', ('r',),
                 [cont('r', '&', 'r', 'ring'),
                  tok('p', '&uniq', 'Prod', 'shr', ('r',)), val('v')]), None),
        main([
            SLetNew('two', 'Two'),
            SLetCall('pt', 'producer', [B('&', P('two', 'a'))]),
            SLetCall(None, 'feed', [B('&', P('two', 'b')), B('&uniq', 'pt'), V()]),  # <- rejects
        ]),
    ]), "REJECT")

# ATK-drop-order-rebind — R7: at scope end t2 is reached while (t2,uniq,e1) live.
add("ATK-drop-order-rebind", Program("ATKdrop", funcs=[main([
    SLetNew('t1', 'table'),
    SLetCall('e1', 'entry', [B('&uniq', 't1'), V()]),
    SLetCall(None, 'fill', [O('e1'), V()]),
    SLetNew('t2', 'table'),
    SLetCall('e2', 'entry', [B('&uniq', 't2'), V()]),
    SRebind('e1', 'e2'),                                          # scope end <- rejects
])]), "REJECT")

# ATK-destructure-env — R11: struct patterns never destructure confined values.
add("ATK-destructure-env-struct", Program("ATKdestruct",
    types=[TypeDecl('Env', True, 'shr', ('e',), fields=[
        Field('lut', borrow_mode='&', borrow_region='e', region_args=('e',))])],
    funcs=[
        Func(Sig('fdes', ('a',), [cont('tbl', '&', 'a', 'table')]),
             [
                SLetConstruct('env', 'Env', None, [('lut', B('&', 'tbl'))]),
                SMatch(Place('env'), TypeInfo('Env', True, 'shr', ('e',)), [
                    MatchArm('Env', None, True, [                # struct pattern <- rejects
                        SLetCall('v', 'get', [B('&', 'tbl'), V()])]),
                ]),
             ]),
    ]), "REJECT")

# ATK-entryresult — R11 flagship: both variants pack one same-kind token; join agrees.
add("ATK-entryresult-occ-vac", Program("ATKentryresult",
    types=[TypeDecl('EntryResult', True, 'uniq', ('t',), is_enum=True, variants=[
        Variant('Occ', [Field('f', confined=True, kind='uniq',
                              type_name='OccTok', region_args=('t',))]),
        Variant('Vac', [Field('f', confined=True, kind='uniq',
                              type_name='VacTok', region_args=('t',))])])],
    local_forms=[Sig('entry', ('t',),
                     [cont('t', '&uniq', 't', 'table'), val('k')],
                     TypeInfo('EntryResult', True, 'uniq', ('t',)),
                     'issues', 'uniq', is_form=True, loanable_form=True)],
    funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('r', 'entry', [B('&uniq', 't'), V()]),
        SMatch(Place('r'), TypeInfo('EntryResult', True, 'uniq', ('t',)), [
            MatchArm('Occ', 'o', False, [SLetCall(None, 'replace', [O('o'), V()])]),
            MatchArm('Vac', 'v0', False, [SLetCall(None, 'fill', [O('v0'), V()])]),
        ]),
    ])]), "ACCEPT")

# ATK-trylock — R11: the Busy variant carries zero token fields.
add("ATK-trylock-enum", Program("ATKtrylock",
    types=[TypeDecl('LockResult', True, 'uniq', ('m',), is_enum=True, variants=[
        Variant('Got', [Field('f', confined=True, kind='uniq',
                              type_name='Guard', region_args=('m',))]),
        Variant('Busy', [])])],
    local_forms=[Sig('try_lock', ('m',),
                     [cont('m', '&', 'm', 'mutex')],
                     TypeInfo('LockResult', True, 'uniq', ('m',)),
                     'issues', 'uniq', is_form=True, loanable_form=True)],
    funcs=[
        Func(Sig('ftry', ('a',), [cont('m', '&', 'a', 'mutex')]), [
            SLetCall('r', 'try_lock', [B('&', 'm')]),
            SMatch(Place('r'), TypeInfo('LockResult', True, 'uniq', ('m',)), [
                MatchArm('Got', 'g', False, []),
                MatchArm('Busy', None, False, []),
            ]),
        ]),
    ]), "REJECT")

# ATK-acquire — retry-acquire crosses the loop by RETURN; F arm consumes.
add("ATK-acquire-helper", Program("ATKacquire", funcs=[
    Func(Sig('acquire', ('m',), [cont('m', '&', 'm', 'mutex')],
             TypeInfo('Guard', True, 'uniq', ('m',))),
         [SLoop([
             SLetCall('g', 'try_lock', [B('&', 'm')]),
             SLetCall('acq', 'acquired', [B('&', 'g')]),
             SMatch(Place('acq'), None, [
                 MatchArm('T', None, False, [SReturn('g')]),
                 MatchArm('F', None, False, [SLetCall(None, 'dismiss', [O('g')])]),
             ]),
         ])]),
    Func(Sig('caller_acq', ('a',), [cont('m', '&', 'a', 'mutex')]),
         [SLetCall('g', 'acquire', [B('&', 'm')])]),
]), "ACCEPT")

# self-tie-env-param — R9 self-tie: env by-borrow, no distinct 'e parameter.
add("self-tie-env-param", Program("selftie",
    types=[TypeDecl('Env', True, 'shr', ('e',), fields=[
        Field('lut', borrow_mode='&', borrow_region='e', region_args=('e',))])],
    funcs=[
        Func(Sig('body', ('c', 'e'),
                 [cont('chunk', '&uniq', 'c', 'chunkview'),
                  tok('env', '&', 'Env', 'shr', ('e',))]), None),
    ]), "ACCEPT")

# ATK-next-view — R5 holder-receiver issue: sibling shr entry on c's source place.
add("ATK-next-view-sibling-issue", Program("ATKnextview", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLoop([SLetCall('v', 'next_view', [B('&', 'c')])]),
])]), "ACCEPT")

# ATK-mk-env2 — R9/R10(b): each region ties to one parameter; two shr issues.
add("ATK-mk-env2-multi-source", Program("ATKmkenv2",
    types=[TypeDecl('Env2', True, 'shr', ('r1', 'r2'), fields=[
        Field('lut', borrow_mode='&', borrow_region='r1', region_args=('r1',)),
        Field('dict', borrow_mode='&', borrow_region='r2', region_args=('r2',))])],
    funcs=[
        Func(Sig('mk_env', ('r1', 'r2'),
                 [cont('t1', '&', 'r1', 'table'), cont('t2', '&', 'r2', 'table')],
                 TypeInfo('Env2', True, 'shr', ('r1', 'r2'))),
             [SLetConstruct('env', 'Env2', None,
                            [('lut', B('&', 't1')), ('dict', B('&', 't2'))]),
              SReturn('env')]),
    ]), "ACCEPT")

# ATK-memoizing — R15: an & -receiver op that memoizes fails concurrent-safety.
add("ATK-memoizing-shared-get", Program("ATKmemo",
    local_forms=[Sig('get', ('t',),
                     [cont('t', '&', 't', 'table'), val('k')], TypeInfo('value'),
                     'none', is_form=True, loanable_form=True,
                     address_stable=True, concurrent_safe=False)],  # <- rejects (R15)
    ), "REJECT")

# ATK-let-move — R8: a plain let-move renames the entry's holder to g2.
add("ATK-let-move-holder", Program("ATKletmove", funcs=[
    Func(Sig('fmove', ('a',), [cont('m', '&', 'a', 'mutex')]),
         [SLetCall('g', 'lock', [B('&', 'm')]), SLetMove('g2', 'g')]),
]), "ACCEPT")

# ATK-loop-all-paths — R12: every path ends in break, so the back edge is unreachable.
add("ATK-loop-all-paths-exit", Program("ATKallpaths", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
    SLoop([SLetCall(None, 'fill', [O('e'), V()]), SBreak()]),
])]), "ACCEPT")

# ATK-zero-region — R1: a loanable confined type must carry >=1 region parameter.
add("ATK-zero-region-endpoint", Program("ATKzero",
    types=[TypeDecl('Tok', True, 'uniq', (), fields=[])],
    local_forms=[Sig('producer', (),
                     [cont('q', '&', 'r', 'conc_queue')],
                     TypeInfo('Tok', True, 'uniq', ()),
                     'issues', 'uniq', is_form=True, loanable_form=True)],
    funcs=[
        Func(Sig('escape', (), [], TypeInfo('Tok', True, 'uniq', ())),
             [SLetNew('q', 'conc_queue'),
              SLetCall('p', 'producer', [B('&', 'q')]),           # <- rejects (R1)
              SReturn('p')]),
    ]), "REJECT")


# ==========================================================================
# HOSTILE-REVIEW EXTENSION (m1-hostile-review.json). Every reviewer program is
# frozen here as a corpus expectation. Soundness breaks + fidelity CEs +
# witnesses now REJECT after the repair round; blessed expressiveness idioms
# ACCEPT, boundary probes REJECT.
# ==========================================================================

def loop_exit_match():
    return SMatch(Place('done'), None, [
        MatchArm('T', None, False, [SBreak()]),
        MatchArm('F', None, False, [SLetCall(None, 'noop', [])]),
    ])

_body2 = Func(Sig('body', ('c', 'e'),
              [cont('chunk', '&uniq', 'c', 'buffer'), cont('env', '&', 'e', 'buffer')]), None)

# --- SOUNDNESS BREAKS -----------------------------------------------------

# BREAK-1: helper &uniq-mints its &-shared param -> R6 mode-capability.
add("BREAK-1-mutate-under-cursor-via-shared-helper", Program("B1", funcs=[
    Func(Sig('mutate_helper', ('t',), [cont('t', '&', 't', 'table')]),
         [SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
          SLetCall(None, 'fill', [O('e'), V()])]),
    main([
        SLetNew('t', 'table'),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),
        SLetCall(None, 'mutate_helper', [B('&', 't')]),
        SLetCall('n', 'next', [B('&uniq', 'c')]),
    ]),
]), "REJECT")

add("CONTROL-1-inline-mutate-under-cursor", Program("C1x", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
    SLetCall(None, 'fill', [O('e'), V()]),
    SLetCall('n', 'next', [B('&uniq', 'c')]),
])]), "REJECT")

add("BREAK-2-par-split-and-replicate-same-place", Program("B2", funcs=[_body2,
    Func(Sig('f', ('a',), [cont('data', '&uniq', 'a', 'buffer')]),
         [SPar('for_chunks', [('split', B('&uniq', 'data')),
                              ('replicate', B('&', 'data'))], 'body')])]), "REJECT")

add("BREAK-2b-par-two-split-same-place", Program("B2b", funcs=[
    Func(Sig('body', ('c', 'd'), [cont('x', '&uniq', 'c', 'buffer'),
                                  cont('y', '&uniq', 'd', 'buffer')]), None),
    Func(Sig('f', ('a',), [cont('data', '&uniq', 'a', 'buffer')]),
         [SPar('for_chunks', [('split', B('&uniq', 'data')),
                              ('split', B('&uniq', 'data'))], 'body')])]), "REJECT")

add("BREAK-2c-par-split-whole-replicate-subfield", Program("B2c", funcs=[_body2,
    Func(Sig('f', ('a',), [cont('data', '&uniq', 'a', 'buffer')]),
         [SPar('for_chunks', [('split', B('&uniq', 'data')),
                              ('replicate', B('&', P('data', 'f')))], 'body')])]), "REJECT")

add("BREAK-2d-par-split-subfield-replicate-whole", Program("B2d", funcs=[_body2,
    Func(Sig('f', ('a',), [cont('data', '&uniq', 'a', 'buffer')]),
         [SPar('for_chunks', [('split', B('&uniq', P('data', 'f'))),
                              ('replicate', B('&', 'data'))], 'body')])]), "REJECT")

# BREAK-3: helper returns a uniq Entry minted from a &-shared table -> at the
# caller the issue lands where the shr cursor is visible (R5).
add("BREAK-3-helper-returns-uniq-from-shared", Program("B3", funcs=[
    Func(Sig('grab', ('t',), [cont('t', '&', 't', 'table')],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SLetCall('e', 'entry', [B('&uniq', 't'), V()]), SReturn('e')]),
    main([
        SLetNew('t', 'table'),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),
        SLetCall('e', 'grab', [B('&', 't')]),
        SLetCall('n', 'next', [B('&uniq', 'c')]),
        SLetCall(None, 'fill', [O('e'), V()]),
    ]),
]), "REJECT")

add("BREAK-4-par-split-over-preexisting-shr-subplace", Program("B4", funcs=[
    Func(Sig('body', ('c',), [cont('x', '&uniq', 'c', 'table')]), None),
    Func(Sig('f', ('a',), [cont('t', '&', 'a', 'table')]),
         [SLetCall('cur', 'range', [B('&', 't'), V(), V()]),
          SPar('for_chunks', [('split', B('&uniq', 't'))], 'body')])]), "REJECT")

add("REGRESSION-par-distinct-places", Program("Rpar", funcs=[_body2,
    Func(Sig('f', ('a',),
             [cont('data', '&uniq', 'a', 'buffer'), cont('lut', '&', 'a', 'buffer')]),
         [SPar('for_chunks', [('split', B('&uniq', 'data')),
                              ('replicate', B('&', 'lut'))], 'body')])]), "ACCEPT")

add("PROBE-sole-loan-with-next_view-sibling", Program("PV", funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('v', 'next_view', [B('&', 'c')]),
    SLetCall(None, 'delete_at', [B('&uniq', 't'), O('c')]),
    SLetCall('n', 'next_view', [B('&', 'v')]),
])]), "REJECT")

add("PROBE-arm-move-outside-holder", Program("AM", funcs=[
    Func(Sig('f', ('a',), [cont('m', '&', 'a', 'mutex')]),
         [SLetCall('g', 'lock', [B('&', 'm')]),
          SMatch(Place('flag'), None, [
              MatchArm('T', None, False, [SLetMove('x', 'g')]),
              MatchArm('F', None, False, []),
          ])])]), "REJECT")

# S1a: region-parameter aliasing -- evil(&uniq t, &uniq t); two &uniq mints of
# the same place in one statement (R6 statement-local disjointness).
add("S1a-region-alias-bare", Program("S1a", funcs=[
    Func(Sig('evil', ('a', 'b'),
             [cont('x', '&uniq', 'a', 'table'), cont('y', '&uniq', 'b', 'table')]),
         [SLetCall('e', 'entry', [B('&uniq', 'x'), V()]),
          SLetCall(None, 'insert', [B('&uniq', 'y'), V(), V()]),
          SLetCall(None, 'fill', [O('e'), V()])]),
    main([SLetNew('t', 'table'),
          SLetCall(None, 'evil', [B('&uniq', 't'), B('&uniq', 't')])]),
]), "REJECT")

# --- SPEC-FIDELITY COUNTEREXAMPLES ---------------------------------------

add("CE1-confined-source-dropped-under-live-loan", Program("CE1",
    types=[TypeDecl('Env', True, 'shr', ('e',), fields=[
        Field('lut', borrow_mode='&', borrow_region='e', region_args=('e',))])],
    funcs=[main([
        SLetNew('t', 'table'),
        SLetConstruct('h0', 'Env', None, [('lut', B('&', 't'))]),
        SLetMove('hx', 'h0'),
        SLetNew('t2', 'table'),
        SBlock([
            SLetCall('e3', 'entry', [B('&uniq', 't2'), V()]),
            SLetConstruct('env2', 'Env', None, [('lut', B('&', 'e3'))]),
            SRebind('h0', 'env2'),
        ]),
        SLetCall(None, 'noop', []),
    ])]), "REJECT")

_stash = Sig('stash', region_params=('t',),
             params=[tok('e', 'own', 'Entry', 'uniq', ('t',))],
             result=None, form_clause='none', is_form=True, loanable_form=True)
add("CE2-own-confined-arg-clause-none-stays-live", Program("CE2",
    local_forms=[_stash], funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('e', 'entry', [B('&uniq', 't'), V()]),
        SLetCall(None, 'stash', [O('e')]),
        SLetCall(None, 'fill', [O('e')]),
    ])]), "REJECT")

add("CE3-confined-scrutinee-with-none-annotation", Program("CE3", funcs=[
    Func(Sig('fm', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SMatch(Place('g'), None, [MatchArm(None, None, False, [SLetCall(None, 'noop', [])])]),
    ]),
]), "REJECT")

add("CE4-consumed-holder-into-replicate-slot", Program("CE4", funcs=[
    Func(Sig('fp', ('a',),
             [cont('m', '&', 'a', 'mutex'), cont('data', '&uniq', 'a', 'buffer')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SLetCall(None, 'dismiss', [O('g')]),
        SPar('for_chunks', [('split', B('&uniq', 'data')),
                            ('replicate', B('&', 'g'))], 'body'),
    ]),
]), "REJECT")

add("CE5-zero-region-loanable-declared-never-called", Program("CE5",
    types=[TypeDecl('Tok', True, 'uniq', (), fields=[])],
    local_forms=[Sig('mk_tok', region_params=(),
                     params=[cont('q', '&', 'r', 'conc_queue')],
                     result=TypeInfo('Tok', True, 'uniq', ()),
                     form_clause='issues', clause_kind='uniq',
                     is_form=True, loanable_form=True)],
    funcs=[main([SLetNew('q', 'conc_queue')])]), "REJECT")

add("CE6-selftie-origin-into-distinct-slot", Program("CE6",
    types=[TypeDecl('Env', True, 'shr', ('e',), fields=[
        Field('lut', borrow_mode='&', borrow_region='e', region_args=('e',))])],
    funcs=[
        Func(Sig('taker', ('e',),
                 [cont('tbl', '&', 'e', 'table'), tok('env', '&', 'Env', 'shr', ('e',))]), None),
        Func(Sig('fwd', ('c', 'e'),
                 [cont('chunk', '&uniq', 'c', 'chunkview'),
                  tok('env', '&', 'Env', 'shr', ('e',)),
                  cont('other', '&', 'z', 'table')]),
             [SLetCall(None, 'taker', [B('&', 'other'), B('&', 'env')])]),
    ]), "REJECT")

# --- R12 WITNESSES --------------------------------------------------------

add("W-D-liveness-only-divergence", Program("WD", funcs=[main([
    SLetNew('s', 'seq'),
    SLetNew('x', 'owned'),
    SMatch(Place('flag'), None, [
        MatchArm('T', None, False, [SLetCall(None, 'push', [B('&uniq', 's'), O('x')])]),
        MatchArm('F', None, False, [SLetCall(None, 'push', [B('&uniq', 's'), V()])]),
    ]),
])]), "REJECT")

add("W-D2-holder-name-swap", Program("WD2", funcs=[
    Func(Sig('fswap', ('a',),
             [cont('m1', '&', 'a', 'mutex'), cont('m2', '&', 'a', 'mutex')]), [
        SLetCall('gA', 'lock', [B('&', 'm1')]),
        SLetCall('gB', 'lock', [B('&', 'm2')]),
        SMatch(Place('flag'), None, [
            MatchArm('T', None, False, [SLetCall(None, 'noop', [])]),
            MatchArm('F', None, False, [
                SLetMove('gT', 'gA'), SRebind('gA', 'gB'), SRebind('gB', 'gT'),
            ]),
        ]),
    ]),
]), "REJECT")

_snap = Sig('snap', region_params=('t',), params=[cont('t', '&', 't', 'table')],
            result=TypeInfo('SnapTok', True, 'uniq', ('t',)),
            form_clause='issues', clause_kind='uniq', is_form=True, loanable_form=True)
add("W-F-uniq-issue-through-and-receiver", Program("WF", local_forms=[_snap],
    funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),
        SLetCall('s', 'snap', [B('&', 't')]),
        SLetCall('n', 'next', [B('&uniq', 'c')]),
    ])]), "REJECT")

# W-G isolates the R5 unbound-region clause: a bare forged construction that
# leaves a region unbound and is never passed to a form op (so the R10a form-
# identity brand added in this repair round does NOT reach it). R5's
# "no captureless instantiation" is the SOLE guard here.
add("W-G-forged-region-construction-bare", Program("WG",
    types=[TypeDecl('MaybeTok', True, 'uniq', ('r',), is_enum=True, variants=[
        Variant('Some', [Field('f', confined=True, kind='uniq',
                               type_name='Entry', region_args=('r',))]),
        Variant('None', [])])],
    funcs=[main([SLetConstruct('none', 'MaybeTok', 'None', [])])]), "REJECT")

# A1b directly exercises R13 (A1 itself rejects at the R9 pre-pass before any
# body runs). The signature is well-formed (result region 't ties to t), but the
# body returns a token whose source is a LOCAL table t2 -- a locally introduced
# region that is not a parameter of the enclosing function.
add("A1b-escape-by-return-body-R13", Program("A1b", funcs=[
    Func(Sig('leak_body', ('t',), [cont('t', '&', 't', 'table')],
             TypeInfo('Entry', True, 'uniq', ('t',))),
         [SLetNew('t2', 'table'),
          SLetCall('e', 'entry', [B('&uniq', 't2'), V()]),
          SReturn('e')]),
]), "REJECT")

# --- EXPRESSIVENESS PROBES (catalog idioms) -------------------------------

def _wc_types():
    return [TypeDecl('WCEntry', True, 'uniq', ('t',), is_enum=True, variants=[
        Variant('Occ', [Field('f', confined=True, kind='uniq', type_name='OccTok', region_args=('t',))]),
        Variant('Vac', [Field('f', confined=True, kind='uniq', type_name='VacTok', region_args=('t',))])])]

def _wc_forms():
    return [
        Sig('wc_entry', ('t',), [cont('t', '&uniq', 't', 'table'), val('k')],
            TypeInfo('WCEntry', True, 'uniq', ('t',)), 'issues', 'uniq', is_form=True, loanable_form=True),
        Sig('incr', ('t',), [tok('o', '&uniq', 'OccTok', 'uniq', ('t',))],
            None, 'none', None, is_form=True, loanable_form=True),
        Sig('fill_slot', ('t',), [tok('v', 'own', 'VacTok', 'uniq', ('t',)), val('x')],
            None, 'consumes', None, is_form=True, loanable_form=True),
    ]

add("E1-wordcount-entry-loop", Program("E1", types=_wc_types(), local_forms=_wc_forms(),
    funcs=[main([
        SLetNew('t', 'table'),
        SLoop([
            SLetCall('k', 'next_k', []),
            SLetCall('r', 'wc_entry', [B('&uniq', 't'), V()]),
            SMatch(Place('r'), TypeInfo('WCEntry', True, 'uniq', ('t',)), [
                MatchArm('Occ', 'o', False, [SLetCall(None, 'incr', [B('&uniq', 'o')])]),
                MatchArm('Vac', 'v0', False, [SLetCall(None, 'fill_slot', [O('v0'), V()])]),
            ]),
            loop_exit_match(),
        ]),
        SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
    ])]), "ACCEPT")

add("E1b-wordcount-vacant-arm-empty", Program("E1b", types=_wc_types(), local_forms=_wc_forms(),
    funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('r', 'wc_entry', [B('&uniq', 't'), V()]),
        SMatch(Place('r'), TypeInfo('WCEntry', True, 'uniq', ('t',)), [
            MatchArm('Occ', 'o', False, [SLetCall(None, 'incr', [B('&uniq', 'o')])]),
            MatchArm('Vac', 'v0', False, []),
        ]),
        SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
    ])]), "ACCEPT")

add("E1c-helper-token-only", Program("E1c", funcs=[
    Func(Sig('bump_bad', ('u',), [tok('o', 'own', 'OccTok', 'uniq', ('u',))]),
         [SLetCall(None, 'replace', [O('o'), V()])]),
]), "REJECT")

add("E1d-helper-threads-container", Program("E1d", types=_wc_types(), local_forms=_wc_forms(),
    funcs=[
        Func(Sig('bump', ('u',),
                 [cont('t', '&', 'u', 'table'), tok('o', 'own', 'OccTok', 'uniq', ('u',))]),
             [SLetCall(None, 'replace', [O('o'), V()])]),
        main([
            SLetNew('t', 'table'),
            SLetCall('r', 'wc_entry', [B('&uniq', 't'), V()]),
            SMatch(Place('r'), TypeInfo('WCEntry', True, 'uniq', ('t',)), [
                MatchArm('Occ', 'o', False, [SLetCall(None, 'bump', [B('&', 't'), O('o')])]),
                MatchArm('Vac', 'v0', False, [SLetCall(None, 'fill_slot', [O('v0'), V()])]),
            ]),
        ]),
    ]), "ACCEPT")

add("E2a-btree-sealed-two-key-op", Program("E2a",
    local_forms=[Sig('rebalance', (), [cont('p', '&uniq', 'p', 'pool'), val('i'), val('j')],
                     None, 'none', None, is_form=True, loanable_form=True)],
    funcs=[main([
        SLetNew('pool', 'pool'),
        SLetCall(None, 'rebalance', [B('&uniq', 'pool'), V(), V()]),
        SLetCall(None, 'rebalance', [B('&uniq', 'pool'), V(), V()]),
    ])]), "ACCEPT")

_np = TypeDecl('NodePair', False, fields=[Field('a', type_name='seq'), Field('b', type_name='seq')])
_move_elems = Sig('move_elems', (),
                  [cont('x', '&uniq', 'x', 'seq'), cont('y', '&uniq', 'y', 'seq'), val('n')],
                  None, 'none', None, is_form=True, loanable_form=True)
add("E2a2-btree-disjoint-node-borrows", Program("E2a2", types=[_np], local_forms=[_move_elems],
    funcs=[main([
        SLetNew('nodes', 'NodePair'),
        SLetCall(None, 'move_elems', [B('&uniq', P('nodes', 'a')), B('&uniq', P('nodes', 'b')), V()]),
        SLetCall(None, 'push', [B('&uniq', P('nodes', 'a')), V()]),
    ])]), "ACCEPT")

add("E2b-btree-take-and-reinsert", Program("E2b",
    local_forms=[
        Sig('take_node', (), [cont('p', '&uniq', 'p', 'pool'), val('i')],
            TypeInfo('seq'), 'none', None, is_form=True, loanable_form=True),
        Sig('put_node', (), [cont('p', '&uniq', 'p', 'pool'), val('i'), val('n')],
            None, 'none', None, is_form=True, loanable_form=True),
    ],
    funcs=[main([
        SLetNew('pool', 'pool'),
        SLetCall('na', 'take_node', [B('&uniq', 'pool'), V()]),
        SLetCall('nb', 'take_node', [B('&uniq', 'pool'), V()]),
        SLetCall('x', 'pop', [B('&uniq', 'na')]),
        SLetCall(None, 'push', [B('&uniq', 'nb'), V()]),
        SLetCall(None, 'put_node', [B('&uniq', 'pool'), V(), O('na')]),
        SLetCall(None, 'put_node', [B('&uniq', 'pool'), V(), O('nb')]),
    ])]), "ACCEPT")

add("E2c-btree-two-entry-tokens", Program("E2c", funcs=[main([
    SLetNew('pool', 'table'),
    SLetCall('e1', 'entry', [B('&uniq', 'pool'), V()]),
    SLetCall('e2', 'entry', [B('&uniq', 'pool'), V()]),
    SLetCall(None, 'fill', [O('e1'), V()]),
    SLetCall(None, 'fill', [O('e2'), V()]),
])]), "REJECT")

# E2d: same-statement aliased &uniq mints -- now closed by the statement-local
# accumulator (was an out-of-lens PROBE ACCEPT before the repair).
add("E2d-aliased-two-uniq-one-statement", Program("E2d", types=[_np], local_forms=[_move_elems],
    funcs=[main([
        SLetNew('nodes', 'NodePair'),
        SLetCall(None, 'move_elems', [B('&uniq', P('nodes', 'a')), B('&uniq', P('nodes', 'a')), V()]),
    ])]), "REJECT")

def _view_forms():
    return [Sig('view_get', ('t',), [tok('v', '&', 'View', 'shr', ('t',))],
                TypeInfo('value'), 'none', None, is_form=True, loanable_form=True)]

add("E3a-cursor-scan-view-loop", Program("E3a", local_forms=_view_forms(), funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLoop([
        SLetCall('v', 'next_view', [B('&', 'c')]),
        SLetCall('x', 'view_get', [B('&', 'v')]),
        loop_exit_match(),
    ]),
    SLetCall('w', 'get', [B('&', 't'), V()]),
])]), "ACCEPT")

add("E3b-insert-under-live-view", Program("E3b", local_forms=_view_forms(), funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('v', 'next_view', [B('&', 'c')]),
    SLetCall(None, 'insert', [B('&uniq', 't'), V(), V()]),
    SLetCall('x', 'view_get', [B('&', 'v')]),
])]), "REJECT")

add("E3c-two-views-coexist", Program("E3c", local_forms=_view_forms(), funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('v1', 'next_view', [B('&', 'c')]),
    SLetCall('v2', 'next_view', [B('&', 'c')]),
    SLetCall('x', 'view_get', [B('&', 'v1')]),
    SLetCall('y', 'view_get', [B('&', 'v2')]),
])]), "ACCEPT")

add("E3d-cursor-consumed-under-view", Program("E3d", local_forms=_view_forms(), funcs=[main([
    SLetNew('t', 'table'),
    SLetCall('c', 'range', [B('&', 't'), V(), V()]),
    SLetCall('v', 'next_view', [B('&', 'c')]),
    SLetCall(None, 'delete_at', [B('&uniq', 't'), O('c')]),
    SLetCall('x', 'view_get', [B('&', 'v')]),
])]), "REJECT")

add("E3e-helper-takes-view", Program("E3e", local_forms=_view_forms(), funcs=[
    Func(Sig('view_helper', ('t',), [tok('v', '&', 'View', 'shr', ('t',))]),
         [SLetCall('x', 'view_get', [B('&', 'v')])]),
    main([
        SLetNew('t', 'table'),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),
        SLetCall('v', 'next_view', [B('&', 'c')]),
        SLetCall(None, 'view_helper', [B('&', 'v')]),
    ]),
]), "ACCEPT")

add("E4a-trylock-inline-loop", Program("E4a", funcs=[
    Func(Sig('f4a', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLoop([
            SLetCall('g', 'try_lock', [B('&', 'm')]),
            SLetCall('acq', 'acquired', [B('&', 'g')]),
            SMatch(Place('acq'), None, [
                MatchArm('T', None, False, [
                    SLetCall('pv', 'predicate', [B('&', 'g')]),
                    SLetCall(None, 'unlock_via', [B('&', 'm'), O('g')]),
                    SBreak(),
                ]),
                MatchArm('F', None, False, [SLetCall(None, 'dismiss', [O('g')])]),
            ]),
        ]),
    ]),
]), "ACCEPT")

add("E4b-acquire-helper-then-use", Program("E4b", funcs=[
    Func(Sig('acquire', ('m',), [cont('m', '&', 'm', 'mutex')],
             TypeInfo('Guard', True, 'uniq', ('m',))),
         [SLoop([
             SLetCall('g', 'try_lock', [B('&', 'm')]),
             SLetCall('acq', 'acquired', [B('&', 'g')]),
             SMatch(Place('acq'), None, [
                 MatchArm('T', None, False, [SReturn('g')]),
                 MatchArm('F', None, False, [SLetCall(None, 'dismiss', [O('g')])]),
             ]),
         ])]),
    Func(Sig('use_it', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'acquire', [B('&', 'm')]),
        SLetCall('pv', 'predicate', [B('&', 'g')]),
        SLetCall(None, 'unlock_via', [B('&', 'm'), O('g')]),
    ]),
]), "ACCEPT")

add("E4c-break-carry-guard", Program("E4c", funcs=[
    Func(Sig('f4c', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLoop([
            SLetCall('g', 'try_lock', [B('&', 'm')]),
            SLetCall('acq', 'acquired', [B('&', 'g')]),
            SMatch(Place('acq'), None, [
                MatchArm('T', None, False, [SBreak()]),
                MatchArm('F', None, False, [SLetCall(None, 'dismiss', [O('g')])]),
            ]),
        ]),
        SLetCall('pv', 'predicate', [B('&', 'g')]),
    ]),
]), "REJECT")

add("E5a-sharded-map-lock-then-entry", Program("E5a",
    types=[TypeDecl('Shard', False, fields=[
               Field('m', type_name='mutex'), Field('t', type_name='table')]),
           TypeDecl('Shards', False, fields=[
               Field('s0', type_name='Shard'), Field('s1', type_name='Shard')])],
    funcs=[main([
        SLetNew('sh', 'Shards'),
        SLetCall('h', 'next_k', []),
        SMatch(Place('h'), None, [
            MatchArm('S0', None, False, [
                SLetCall('g', 'lock', [B('&', P('sh', 's0', 'm'))]),
                SLetCall('e', 'entry', [B('&uniq', P('sh', 's0', 't')), V()]),
                SLetCall(None, 'fill', [O('e'), V()]),
            ]),
            MatchArm('S1', None, False, [
                SLetCall('g', 'lock', [B('&', P('sh', 's1', 'm'))]),
                SLetCall('e', 'entry', [B('&uniq', P('sh', 's1', 't')), V()]),
                SLetCall(None, 'fill', [O('e'), V()]),
            ]),
        ]),
        SLetCall(None, 'insert', [B('&uniq', P('sh', 's0', 't')), V(), V()]),
    ])]), "ACCEPT")

add("E5b-guard-statement-update", Program("E5b",
    local_forms=[Sig('gd_update', ('m',),
                     [tok('g', '&uniq', 'Guard', 'uniq', ('m',)), val('k'), val('x')],
                     None, 'none', None, is_form=True, loanable_form=True)],
    funcs=[Func(Sig('f5b', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SLetCall(None, 'gd_update', [B('&uniq', 'g'), V(), V()]),
        SLetCall(None, 'gd_update', [B('&uniq', 'g'), V(), V()]),
    ])]), "ACCEPT")

add("E5c-entry-token-through-live-guard", Program("E5c",
    local_forms=[
        Sig('gd_entry', ('m',), [tok('g', '&', 'Guard', 'uniq', ('m',)), val('k')],
            TypeInfo('GEntry', True, 'uniq', ('m',)), 'issues_on_source', 'uniq',
            is_form=True, loanable_form=True),
        Sig('gd_fill', ('m',), [tok('e', 'own', 'GEntry', 'uniq', ('m',)), val('x')],
            None, 'consumes', None, is_form=True, loanable_form=True),
    ],
    funcs=[Func(Sig('f5c', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SLetCall('e', 'gd_entry', [B('&', 'g'), V()]),
        SLetCall(None, 'gd_fill', [O('e'), V()]),
    ])]), "REJECT")

add("E5d-lock-entry-reissues", Program("E5d",
    local_forms=[
        Sig('lock_entry', ('m',),
            [cont('m', '&', 'm', 'mutex'), tok('g', 'own', 'Guard', 'uniq', ('m',)), val('k')],
            TypeInfo('GEntry', True, 'uniq', ('m',)), 'reissues', 'uniq',
            is_form=True, loanable_form=True),
        Sig('gd_fill', ('m',), [tok('e', 'own', 'GEntry', 'uniq', ('m',)), val('x')],
            None, 'consumes', None, is_form=True, loanable_form=True),
    ],
    funcs=[Func(Sig('f5d', ('a',), [cont('m', '&', 'a', 'mutex')]), [
        SLetCall('g', 'lock', [B('&', 'm')]),
        SLetCall('e', 'lock_entry', [B('&', 'm'), O('g'), V()]),
        SLetCall(None, 'gd_fill', [O('e'), V()]),
    ])]), "ACCEPT")

def _bufread_forms(extra=()):
    base = [
        Sig('fill_buf', ('r',), [cont('r', '&', 'r', 'reader')],
            TypeInfo('BView', True, 'shr', ('r',)), 'issues', 'shr', is_form=True, loanable_form=True),
        Sig('bview_len', ('r',), [tok('v', '&', 'BView', 'shr', ('r',))],
            TypeInfo('value'), 'none', None, is_form=True, loanable_form=True),
        Sig('consume_amt', (), [cont('r', '&uniq', 'r', 'reader'), val('n')],
            None, 'none', None, is_form=True, loanable_form=True),
    ]
    return base + list(extra)

add("E6a-bufread-scoped-view", Program("E6a", local_forms=_bufread_forms(), funcs=[main([
    SLetNew('rd', 'reader'),
    SLoop([
        SBlock([
            SLetCall('v', 'fill_buf', [B('&', 'rd')]),
            SLetCall('n', 'bview_len', [B('&', 'v')]),
        ]),
        SLetCall(None, 'consume_amt', [B('&uniq', 'rd'), V()]),
        loop_exit_match(),
    ]),
])]), "ACCEPT")

add("E6b-bufread-consuming-advance", Program("E6b",
    local_forms=_bufread_forms(extra=[
        Sig('consume_via', ('r',),
            [cont('r', '&uniq', 'r', 'reader'), tok('v', 'own', 'BView', 'shr', ('r',)), val('n')],
            None, 'consumes', None, is_form=True, loanable_form=True),
    ]),
    funcs=[main([
        SLetNew('rd', 'reader'),
        SLoop([
            SLetCall('v', 'fill_buf', [B('&', 'rd')]),
            SLetCall('n', 'bview_len', [B('&', 'v')]),
            SLetCall(None, 'consume_via', [B('&uniq', 'rd'), O('v'), V()]),
            loop_exit_match(),
        ]),
    ])]), "ACCEPT")

add("E6c-bufread-nll-ordering", Program("E6c", local_forms=_bufread_forms(), funcs=[main([
    SLetNew('rd', 'reader'),
    SLoop([
        SLetCall('v', 'fill_buf', [B('&', 'rd')]),
        SLetCall('n', 'bview_len', [B('&', 'v')]),
        SLetCall(None, 'consume_amt', [B('&uniq', 'rd'), V()]),
        loop_exit_match(),
    ]),
])]), "REJECT")

add("E7a-mut-scan-via-uniq-cursor", Program("E7a",
    local_forms=[
        Sig('mut_range', ('t',), [cont('t', '&uniq', 't', 'table'), val('lo'), val('hi')],
            TypeInfo('MutCursor', True, 'uniq', ('t',)), 'issues', 'uniq', is_form=True, loanable_form=True),
        Sig('write_current', ('t',), [tok('c', '&uniq', 'MutCursor', 'uniq', ('t',)), val('x')],
            None, 'none', None, is_form=True, loanable_form=True),
        Sig('advance', ('t',), [tok('c', '&uniq', 'MutCursor', 'uniq', ('t',))],
            TypeInfo('option'), 'none', None, is_form=True, loanable_form=True),
    ],
    funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('c', 'mut_range', [B('&uniq', 't'), V(), V()]),
        SLoop([
            SLetCall(None, 'write_current', [B('&uniq', 'c'), V()]),
            SLetCall('n', 'advance', [B('&uniq', 'c')]),
            loop_exit_match(),
        ]),
    ])]), "ACCEPT")

add("E7b-uniq-view-from-shr-cursor", Program("E7b",
    local_forms=[Sig('next_mut_view', ('t',), [tok('c', '&', 'Cursor', 'shr', ('t',))],
                     TypeInfo('MutView', True, 'uniq', ('t',)), 'issues_on_source', 'uniq',
                     is_form=True, loanable_form=True)],
    funcs=[main([
        SLetNew('t', 'table'),
        SLetCall('c', 'range', [B('&', 't'), V(), V()]),
        SLetCall('v', 'next_mut_view', [B('&', 'c')]),
    ])]), "REJECT")
