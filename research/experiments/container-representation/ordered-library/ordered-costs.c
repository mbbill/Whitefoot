#if !defined(_WIN32)
#define _POSIX_C_SOURCE 200809L
#endif
#include <assert.h>
#include <inttypes.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#else
#include <time.h>
#endif

/* This file is also compiled as six specialized control translation units.
 * Keeping the native algorithms here makes the explicit Makefile their only
 * caller. Public operations and callbacks alone use PUBLIC in retained mode. */
#define NOINLINE __attribute__((noinline))
#if defined(RETAIN_PUBLIC)
#define PUBLIC NOINLINE
#define CONTRACT "retained"
#else
#define PUBLIC
#define CONTRACT "normal"
#endif
enum { FANOUT = 16, MAX_PAIRS = 15, MIN_PAIRS = 7, CEILING = 8192 };
typedef struct { uint64_t ordered, sum, parity, consumed; } Digest;
typedef struct { uint64_t words[32]; } Record;
__attribute__((malloc, returns_nonnull)) void *wf_cost_allocate(uint64_t bytes);
void wf_cost_release(void *pointer);
static __attribute__((unused)) void require(bool condition, const char *message) {
    if (!condition) { fprintf(stderr, "ordered library costs: %s\n", message); exit(1); }
}
static uint64_t mix(uint64_t value) {
    value = (value ^ (value >> 30)) * UINT64_C(13787848793156543929);
    value = (value ^ (value >> 27)) * UINT64_C(10723151780598845931);
    return value ^ (value >> 31);
}
static void ordered(Digest *digest, uint64_t value) {
    digest->ordered = digest->ordered * UINT64_C(131) + value;
}
static void consumed(Digest *digest, uint64_t value) {
    digest->sum += value; digest->parity ^= mix(value); ++digest->consumed;
}
static uint64_t finish(Digest digest) {
    return mix(digest.ordered) ^ mix(digest.sum) ^ digest.parity
        ^ digest.consumed * UINT64_C(11400714819323198485);
}
static uint64_t stream_rank(uint64_t index, uint64_t count) {
    return index * UINT64_C(73) % count;
}
static uint64_t stream_key(uint64_t rank) { return rank * 2 + 1; }

#if defined(CONTROL_TEMPLATE)
#define JOIN_INNER(a, b) a##b
#define JOIN(a, b) JOIN_INNER(a, b)
#define FN(name) JOIN(PREFIX, name)
#if defined(WIDE_PAIR)
typedef Record Value;
#define WORDS 32
#else
typedef uint64_t Value;
#define WORDS 1
#endif
typedef struct { uint64_t key; Value value; } Pair;
#if defined(SOURCE_CONTROL)
typedef struct { uint8_t variant, reason; Pair pair; } Put;
typedef struct { uint8_t present; Pair pair; } Taken;
typedef struct { uint8_t error; uint64_t value; } Lookup;
#define PUT_STATUS(result) ((result).variant ? 1 + (result).reason : 0)
#define LOOKUP_PRESENT(result) (!(result).error)
#define KEY_ARGUMENT(value) (&(const uint64_t){value})
#else
typedef struct { uint64_t status; Pair pair; } Put;
typedef struct { uint64_t present; Pair pair; } Taken;
typedef struct { uint64_t present, value; } Lookup;
#define PUT_STATUS(result) ((result).status)
#define LOOKUP_PRESENT(result) ((result).present)
#define KEY_ARGUMENT(value) (value)
#endif
static __attribute__((unused)) Put FN(_put_outcome)(uint64_t status, Pair pair) {
    Put result;
#if defined(SOURCE_CONTROL)
    result.variant = status != 0;
    if (status != 0) { result.reason = (uint8_t)(status - 1); result.pair = pair; }
#else
    result.status = status;
    if (status != 0) result.pair = pair;
#endif
    return result;
}
static __attribute__((unused)) Put FN(_inserted)(void) {
    Put result;
#if defined(SOURCE_CONTROL)
    result.variant = 0;
#else
    result.status = 0;
#endif
    return result;
}
static Taken FN(_absent)(void) { Taken result; result.present = 0; return result; }
#if defined(STRUCTURAL_AUDIT)
static uint64_t structural_counts[11];
#define STRUCTURAL_COUNT(index) (++structural_counts[index])
#else
#define STRUCTURAL_COUNT(index) ((void)0)
#endif
_Static_assert(sizeof(Pair) == (WORDS + 1) * 8, "pair extent");
static PUBLIC int32_t FN(_compare)(const uint64_t *left, const uint64_t *right) {
    return *left < *right ? -7 : *left > *right ? 9 : 0;
}
static PUBLIC Value FN(_make)(uint64_t seed) {
#if defined(WIDE_PAIR)
    Value value;
    for (unsigned i = 0; i < WORDS; ++i) value.words[i] = seed + i;
    return value;
#else
    return seed;
#endif
}
static uint64_t FN(_content)(uint64_t key, const Value *value) {
    uint64_t result = key;
#if defined(WIDE_PAIR)
    for (unsigned i = 0; i < WORDS; ++i) result = result * UINT64_C(131) + value->words[i];
#else
    result = result * UINT64_C(131) + *value;
#endif
    return result;
}
static PUBLIC uint64_t FN(_observe)(Digest *digest, const uint64_t *key, const Value *value) {
    uint64_t content = FN(_content)(*key, value);
    ordered(digest, content); return content;
}
static PUBLIC void FN(_visit)(Digest *digest, const uint64_t *key, const Value *value) {
    ordered(digest, FN(_content)(*key, value));
}
static PUBLIC uint64_t FN(_edit_value)(Digest *digest, const uint64_t *key, Value *value) {
#if defined(WIDE_PAIR)
    for (unsigned i = 0; i < WORDS; ++i) ++value->words[i];
#else
    ++*value;
#endif
    uint64_t content = FN(_content)(*key, value);
    ordered(digest, content); return content;
}
static PUBLIC void FN(_consume)(Digest *digest, uint64_t key, Value value) {
    consumed(digest, FN(_content)(key, &value));
}

#if defined(AVL_CONTROL)
typedef struct Node { struct Node *left, *right; uint64_t height; Pair pair; } Node;
typedef struct { Node *root; uint64_t length; } Map;
_Static_assert(sizeof(Node) == sizeof(Pair) + 24, "native AVL node extent");
#define ROOT(map) ((map)->root)
static uint64_t FN(_height)(Node *node) { return node ? node->height : 0; }
static void FN(_update)(Node *node) {
    uint64_t left = FN(_height)(node->left), right = FN(_height)(node->right);
    node->height = 1 + (left > right ? left : right);
}
static Node *FN(_rotate_left)(Node *node) {
    Node *right = node->right; node->right = right->left; right->left = node;
    FN(_update)(node); FN(_update)(right); return right;
}
static Node *FN(_rotate_right)(Node *node) {
    Node *left = node->left; node->left = left->right; left->right = node;
    FN(_update)(node); FN(_update)(left); return left;
}
static Node *FN(_balance)(Node *node) {
    FN(_update)(node);
    if (FN(_height)(node->left) > FN(_height)(node->right) + 1) {
        if (FN(_height)(node->left->right) > FN(_height)(node->left->left))
            node->left = FN(_rotate_left)(node->left);
        return FN(_rotate_right)(node);
    }
    if (FN(_height)(node->right) > FN(_height)(node->left) + 1) {
        if (FN(_height)(node->right->left) > FN(_height)(node->right->right))
            node->right = FN(_rotate_right)(node->right);
        return FN(_rotate_left)(node);
    }
    return node;
}
static Node *FN(_insert_node)(Node *node, const Pair *pair, bool room, Put *result) {
    if (node == NULL) {
        if (!room) { result->status = 2; result->pair = *pair; return NULL; }
        Node *made = wf_cost_allocate(sizeof *made);
        *made = (Node){NULL, NULL, 1, *pair}; result->status = 0; return made;
    }
    int32_t order = FN(_compare)(&pair->key, &node->pair.key);
    if (order == 0) { result->status = 1; result->pair = node->pair; node->pair = *pair; return node; }
    if (order < 0) node->left = FN(_insert_node)(node->left, pair, room, result);
    else node->right = FN(_insert_node)(node->right, pair, room, result);
    return FN(_balance)(node);
}
static Node *FN(_take_min)(Node *node, Pair *pair) {
    if (node->left == NULL) {
        Node *right = node->right; *pair = node->pair; wf_cost_release(node); return right;
    }
    node->left = FN(_take_min)(node->left, pair); return FN(_balance)(node);
}
static Node *FN(_remove_node)(Node *node, uint64_t key, Taken *result) {
    if (node == NULL) return NULL;
    int32_t order = FN(_compare)(&key, &node->pair.key);
    if (order < 0) node->left = FN(_remove_node)(node->left, key, result);
    else if (order > 0) node->right = FN(_remove_node)(node->right, key, result);
    else {
        result->present = 1; result->pair = node->pair;
        if (node->left == NULL || node->right == NULL) {
            Node *child = node->left ? node->left : node->right;
            wf_cost_release(node); return child;
        }
        node->right = FN(_take_min)(node->right, &node->pair);
    }
    return FN(_balance)(node);
}
static Pair *FN(_find)(Node *node, uint64_t key) {
    while (node != NULL) {
        int32_t order = FN(_compare)(&key, &node->pair.key);
        if (order == 0) return &node->pair;
        node = order < 0 ? node->left : node->right;
    }
    return NULL;
}
static void FN(_each_node)(Node *node, Digest *digest) {
    if (node == NULL) return;
    FN(_each_node)(node->left, digest); FN(_visit)(digest, &node->pair.key, &node->pair.value);
    FN(_each_node)(node->right, digest);
}
static void FN(_range_node)(Node *node, uint64_t lower, uint64_t upper, Digest *digest) {
    if (node == NULL) return;
    int32_t low = FN(_compare)(&node->pair.key, &lower), high = FN(_compare)(&node->pair.key, &upper);
    if (low > 0) FN(_range_node)(node->left, lower, upper, digest);
    if (low >= 0 && high < 0) FN(_visit)(digest, &node->pair.key, &node->pair.value);
    if (high < 0) FN(_range_node)(node->right, lower, upper, digest);
}
static void FN(_free_node)(Node *node, Digest *digest) {
    if (node == NULL) return;
    FN(_free_node)(node->left, digest); FN(_free_node)(node->right, digest);
    FN(_consume)(digest, node->pair.key, node->pair.value); wf_cost_release(node);
}
static PUBLIC Map FN(_new)(void) { return (Map){NULL, 0}; }
static PUBLIC Put FN(_put)(Map *map, uint64_t key, Value value) {
    Put result; Pair pair = {key, value};
    map->root = FN(_insert_node)(map->root, &pair, map->length < CEILING, &result);
    if (result.status == 0) ++map->length;
    return result;
}
static PUBLIC Taken FN(_remove)(Map *map, uint64_t key) {
    Taken result = FN(_absent)(); map->root = FN(_remove_node)(map->root, key, &result);
    if (result.present) --map->length;
    return result;
}
#elif defined(SOURCE_CONTROL)
typedef struct Node Node;
typedef struct { uint64_t length; Node *node; } Link;
typedef struct { Pair pair; Link after; } Entry;
typedef struct { uint64_t count; Entry data[MAX_PAIRS]; } Entries;
struct Node { Link leading; Entries entries; };
typedef struct { Link root; uint64_t length; } Map;
typedef struct { uint8_t present; Entry entry; } Promotion;
#define ROOT(map) ((map)->root.length ? (map)->root.node : NULL)
#define IS_LEAF(node) ((node)->leading.length == 0)
_Static_assert(sizeof(Node) == 15 * sizeof(Pair) + 264, "bundled WF node extent");
static Promotion FN(_none)(void) { Promotion result; result.present = 0; return result; }
static Promotion FN(_some)(Entry entry) { return (Promotion){1, entry}; }
static Node *FN(_box)(Node value) {
    Node *node = wf_cost_allocate(sizeof *node); *node = value; return node;
}
static void FN(_append_link)(Link *destination, Link *source) {
    if (source->length != 0) destination->node = source->node;
    destination->length += source->length; source->length = 0;
}
static Node *FN(_take_link)(Link *link) { --link->length; return link->node; }
static void FN(_place_link)(Link *link, Node *node) { link->node = node; ++link->length; }
static void FN(_insert_entry)(Entries *entries, uint64_t at, Entry entry) {
    memmove(entries->data + at + 1, entries->data + at, (entries->count - at) * sizeof(Entry));
    entries->data[at] = entry; ++entries->count;
}
static Entry FN(_remove_entry)(Entries *entries, uint64_t at) {
    Entry entry = entries->data[at]; --entries->count;
    memmove(entries->data + at, entries->data + at + 1, (entries->count - at) * sizeof(Entry));
    return entry;
}
static uint64_t FN(_position)(Node *node, const uint64_t *key) {
    uint64_t count = node->entries.count;
    for (uint64_t i = 0; i < count; ++i)
        if (FN(_compare)(key, &node->entries.data[i].pair.key) <= 0) return i;
    return count;
}
static bool FN(_replace_node)(Node *node, Pair *offered) {
    uint64_t at = FN(_position)(node, &offered->key);
    if (at < node->entries.count && FN(_compare)(&offered->key, &node->entries.data[at].pair.key) == 0) {
        Pair previous = node->entries.data[at].pair; node->entries.data[at].pair = *offered; *offered = previous; return true;
    }
    if (at == 0) {
        if (node->leading.length > 0) return FN(_replace_node)(node->leading.node, offered);
    } else if (node->entries.data[at - 1].after.length > 0) {
        return FN(_replace_node)(node->entries.data[at - 1].after.node, offered);
    }
    return false;
}
static Promotion FN(_insert_item)(Node *node, uint64_t at, Entry item) {
    if (node->entries.count < MAX_PAIRS) { FN(_insert_entry)(&node->entries, at, item); return FN(_none)(); }
    STRUCTURAL_COUNT(10); /* The public root-promotion arm reclassifies root splits. */
    Entries upper; upper.count = 0;
    for (unsigned offset = 0; offset < 7; ++offset) {
        Entry entry = node->entries.data[--node->entries.count]; upper.data[upper.count++] = entry;
    }
    for (unsigned offset = 0; offset < 3; ++offset) {
        Entry entry = upper.data[offset]; upper.data[offset] = upper.data[6 - offset]; upper.data[6 - offset] = entry;
    }
    Entry promoted = node->entries.data[--node->entries.count];
    if (at <= 7) FN(_insert_entry)(&node->entries, at, item);
    else FN(_insert_entry)(&upper, at - 8, item);
    Link leading; leading.length = 0; FN(_append_link)(&leading, &promoted.after);
    Node right_node = {leading, upper}; Node *right = FN(_box)(right_node);
    FN(_place_link)(&promoted.after, right); return FN(_some)(promoted);
}
static Promotion FN(_insert_node)(Node *node, Pair pair);
static Promotion FN(_insert_link)(Link *link, Pair pair) {
    if (link->length > 0) return FN(_insert_node)(link->node, pair);
    Link after; after.length = 0; return FN(_some)((Entry){pair, after});
}
static Promotion FN(_insert_node)(Node *node, Pair pair) {
    uint64_t at = FN(_position)(node, &pair.key);
    Promotion outcome;
    if (at == 0) outcome = FN(_insert_link)(&node->leading, pair);
    else outcome = FN(_insert_link)(&node->entries.data[at - 1].after, pair);
    if (!outcome.present) return FN(_none)();
    return FN(_insert_item)(node, at, outcome.entry);
}
static PUBLIC Map FN(_new)(void) { Map map; map.root.length = 0; map.length = 0; return map; }
static PUBLIC Put FN(_put)(Map *map, uint64_t key, Value value) {
    Pair pair = {key, value};
    if (map->root.length > 0 && FN(_replace_node)(map->root.node, &pair)) return FN(_put_outcome)(1, pair);
    if (map->length >= CEILING) return FN(_put_outcome)(2, pair);
    Promotion promoted = FN(_insert_link)(&map->root, pair);
    if (promoted.present) {
#if defined(STRUCTURAL_AUDIT)
        if (map->root.length > 0) { ++structural_counts[9]; --structural_counts[10]; }
#endif
        Link leading; leading.length = 0; FN(_append_link)(&leading, &map->root);
        Entries entries; entries.count = 0; entries.data[entries.count++] = promoted.entry;
        Node root_node = {leading, entries}; Node *root = FN(_box)(root_node); FN(_place_link)(&map->root, root);
    }
    ++map->length; return FN(_inserted)();
}
static void FN(_borrow_right)(Node *left, Node *right, Pair *separator) {
    STRUCTURAL_COUNT(IS_LEAF(left) ? 2 : 3);
    Entry first = FN(_remove_entry)(&right->entries, 0);
    Pair pair = first.pair; first.pair = *separator; *separator = pair;
    Link link = first.after; first.after = right->leading; right->leading = link;
    left->entries.data[left->entries.count++] = first;
}
static void FN(_borrow_left)(Node *left, Node *right, Pair *separator) {
    STRUCTURAL_COUNT(IS_LEAF(right) ? 0 : 1);
    Entry last = left->entries.data[--left->entries.count];
    Pair pair = last.pair; last.pair = *separator; *separator = pair;
    Link link = last.after; last.after = right->leading; right->leading = link;
    FN(_insert_entry)(&right->entries, 0, last);
}
static void FN(_merge)(Node *left, Node *right, Entry separator) {
    STRUCTURAL_COUNT(IS_LEAF(left) ? 4 : 5);
    FN(_append_link)(&separator.after, &right->leading);
    left->entries.data[left->entries.count++] = separator;
    memcpy(left->entries.data + left->entries.count, right->entries.data, right->entries.count * sizeof(Entry));
    left->entries.count += right->entries.count; right->entries.count = 0;
    Node empty = *right; (void)empty; wf_cost_release(right);
}
static Promotion FN(_repair_separator)(Link *left, Entry separator) {
    if (left->length == 0) return FN(_some)(separator);
    if (separator.after.length == 0) return FN(_some)(separator);
    Node *right = FN(_take_link)(&separator.after);
    if (left->node->entries.count < MIN_PAIRS) {
        if (right->entries.count > MIN_PAIRS) FN(_borrow_right)(left->node, right, &separator.pair);
        else { FN(_merge)(left->node, right, separator); return FN(_none)(); }
    } else if (right->entries.count < MIN_PAIRS) {
        if (left->node->entries.count > MIN_PAIRS) FN(_borrow_left)(left->node, right, &separator.pair);
        else { FN(_merge)(left->node, right, separator); return FN(_none)(); }
    }
    FN(_place_link)(&separator.after, right); return FN(_some)(separator);
}
static void FN(_repair_at)(Node *node, uint64_t at) {
    Entry separator = FN(_remove_entry)(&node->entries, at); Promotion outcome;
    if (at == 0) outcome = FN(_repair_separator)(&node->leading, separator);
    else outcome = FN(_repair_separator)(&node->entries.data[at - 1].after, separator);
    if (outcome.present) FN(_insert_entry)(&node->entries, at, outcome.entry);
}
static void FN(_repair_child)(Node *node, uint64_t at) {
    if (node->entries.count == 0) return;
    if (at == 0) {
        if (node->leading.length > 0 && node->leading.node->entries.count < MIN_PAIRS) FN(_repair_at)(node, 0);
    } else if (node->entries.data[at - 1].after.length > 0 && node->entries.data[at - 1].after.node->entries.count < MIN_PAIRS) {
        FN(_repair_at)(node, at - 1);
    }
}
static void FN(_contract_root)(Link *root) {
    if (root->length > 0) {
        Node *old = FN(_take_link)(root);
        if (old->entries.count == 0) {
            if (old->leading.length > 0) STRUCTURAL_COUNT(8);
            Node empty = *old; wf_cost_release(old); FN(_append_link)(root, &empty.leading);
        } else FN(_place_link)(root, old);
    }
}
static Pair FN(_take_min)(Node *node) {
    if (node->leading.length == 0) {
        Entry first = FN(_remove_entry)(&node->entries, 0);
        FN(_append_link)(&node->leading, &first.after); return first.pair;
    }
    if (node->leading.node->entries.count > 0) {
        Pair pair = FN(_take_min)(node->leading.node); FN(_repair_child)(node, 0); return pair;
    }
    FN(_contract_root)(&node->leading); return FN(_take_min)(node);
}
static Pair FN(_finish_remove)(Node *node, uint64_t at, Entry item) {
    if (item.after.length == 0) return item.pair;
    if (item.after.node->entries.count > 0) {
        STRUCTURAL_COUNT(7);
        Pair pair = FN(_take_min)(item.after.node), removed = item.pair; item.pair = pair;
        FN(_insert_entry)(&node->entries, at, item); FN(_repair_child)(node, at + 1); return removed;
    }
    FN(_contract_root)(&item.after); return FN(_finish_remove)(node, at, item);
}
static Pair FN(_remove_item)(Node *node, uint64_t at) {
    Entry item = FN(_remove_entry)(&node->entries, at);
    return FN(_finish_remove)(node, at, item);
}
static Taken FN(_remove_node)(Node *node, const uint64_t *key);
static Taken FN(_remove_link)(Link *link, const uint64_t *key) {
    if (link->length > 0) return FN(_remove_node)(link->node, key);
    return FN(_absent)();
}
static Taken FN(_remove_node)(Node *node, const uint64_t *key) {
    uint64_t at = FN(_position)(node, key);
    if (at < node->entries.count && FN(_compare)(key, &node->entries.data[at].pair.key) == 0)
        return (Taken){1, FN(_remove_item)(node, at)};
    Taken outcome;
    if (at == 0) outcome = FN(_remove_link)(&node->leading, key);
    else outcome = FN(_remove_link)(&node->entries.data[at - 1].after, key);
    if (outcome.present) FN(_repair_child)(node, at);
    return outcome;
}
static PUBLIC Taken FN(_remove)(Map *map, const uint64_t *key) {
    Taken outcome = FN(_remove_link)(&map->root, key);
    if (outcome.present) {
        FN(_contract_root)(&map->root);
        if (map->length > 0) --map->length;
    }
    return outcome;
}
static Lookup FN(_lookup_node)(Node *node, const uint64_t *key, Digest *digest);
static Lookup FN(_lookup_link)(Link *link, const uint64_t *key, Digest *digest) {
    if (link->length > 0) return FN(_lookup_node)(link->node, key, digest);
    Lookup result; result.error = 1; return result;
}
static Lookup FN(_lookup_node)(Node *node, const uint64_t *key, Digest *digest) {
    uint64_t at = FN(_position)(node, key);
    if (at < node->entries.count && FN(_compare)(key, &node->entries.data[at].pair.key) == 0)
        return (Lookup){0, FN(_observe)(digest, &node->entries.data[at].pair.key, &node->entries.data[at].pair.value)};
    if (at == 0) return FN(_lookup_link)(&node->leading, key, digest);
    return FN(_lookup_link)(&node->entries.data[at - 1].after, key, digest);
}
static Lookup FN(_edit_node)(Node *node, const uint64_t *key, Digest *digest);
static Lookup FN(_edit_link)(Link *link, const uint64_t *key, Digest *digest) {
    if (link->length > 0) return FN(_edit_node)(link->node, key, digest);
    Lookup result; result.error = 1; return result;
}
static Lookup FN(_edit_node)(Node *node, const uint64_t *key, Digest *digest) {
    uint64_t at = FN(_position)(node, key);
    if (at < node->entries.count && FN(_compare)(key, &node->entries.data[at].pair.key) == 0)
        return (Lookup){0, FN(_edit_value)(digest, &node->entries.data[at].pair.key, &node->entries.data[at].pair.value)};
    if (at == 0) return FN(_edit_link)(&node->leading, key, digest);
    return FN(_edit_link)(&node->entries.data[at - 1].after, key, digest);
}
static void FN(_each_node)(Node *node, Digest *digest);
static void FN(_each_link)(Link *link, Digest *digest) {
    if (link->length > 0) FN(_each_node)(link->node, digest);
}
static void FN(_each_node)(Node *node, Digest *digest) {
    FN(_each_link)(&node->leading, digest);
    uint64_t count = node->entries.count;
    for (uint64_t i = 0; i < count; ++i) {
        FN(_visit)(digest, &node->entries.data[i].pair.key, &node->entries.data[i].pair.value);
        FN(_each_link)(&node->entries.data[i].after, digest);
    }
}
static void FN(_range_node)(Node *node, const uint64_t *lower, const uint64_t *upper, Digest *digest);
static void FN(_range_link)(Link *link, const uint64_t *lower, const uint64_t *upper, Digest *digest) {
    if (link->length > 0) FN(_range_node)(link->node, lower, upper, digest);
}
static void FN(_range_node)(Node *node, const uint64_t *lower, const uint64_t *upper, Digest *digest) {
    uint64_t start = FN(_position)(node, lower);
    if (start == 0) FN(_range_link)(&node->leading, lower, upper, digest);
    else FN(_range_link)(&node->entries.data[start - 1].after, lower, upper, digest);
    uint64_t count = node->entries.count;
    for (uint64_t i = start; i < count; ++i) {
        if (FN(_compare)(&node->entries.data[i].pair.key, upper) >= 0) return;
        FN(_visit)(digest, &node->entries.data[i].pair.key, &node->entries.data[i].pair.value);
        FN(_range_link)(&node->entries.data[i].after, lower, upper, digest);
    }
}
static void FN(_free_node)(Node *node, Digest *digest);
static void FN(_free_link)(Link link, Digest *digest) {
    if (link.length > 0) { Node *node = FN(_take_link)(&link); FN(_free_node)(node, digest); }
}
static void FN(_free_node)(Node *node, Digest *digest) {
    while (node->entries.count != 0) {
        Entry item = node->entries.data[--node->entries.count];
        FN(_consume)(digest, item.pair.key, item.pair.value); FN(_free_link)(item.after, digest);
    }
    Node empty = *node; wf_cost_release(node); FN(_free_link)(empty.leading, digest);
}
static PUBLIC Lookup FN(_lookup)(Map *map, const uint64_t *key, Digest *digest) { return FN(_lookup_link)(&map->root, key, digest); }
static PUBLIC Lookup FN(_edit)(Map *map, const uint64_t *key, Digest *digest) { return FN(_edit_link)(&map->root, key, digest); }
static PUBLIC void FN(_each)(Map *map, Digest *digest) { FN(_each_link)(&map->root, digest); }
static PUBLIC void FN(_range)(Map *map, const uint64_t *lower, const uint64_t *upper, Digest *digest) {
    if (FN(_compare)(lower, upper) < 0) FN(_range_link)(&map->root, lower, upper, digest);
}
static PUBLIC void FN(_free)(Map map, Digest *digest) { FN(_free_link)(map.root, digest); }
#else
/* Direct C packs a leaf flag beside a 32-bit count and derives the number of
 * live child pointers. It uses ordinary top-down split and deletion repair. */
typedef struct Node { uint32_t count; bool leaf; struct Node *child[FANOUT]; Pair pairs[MAX_PAIRS]; } Node;
typedef struct { Node *root; uint64_t length; } Map;
_Static_assert(sizeof(Node) == 15 * sizeof(Pair) + 136, "direct B-tree node extent");
#define ROOT(map) ((map)->root)
#define IS_LEAF(n) ((n)->leaf)
#define SET_CHILDREN(n, c) ((n)->leaf = (c) == 0)
#define CHILDREN(n) (IS_LEAF(n) ? 0 : (uint64_t)(n)->count + 1)
static Node *FN(_node_new)(bool leaf) {
    Node *node = wf_cost_allocate(sizeof *node); node->count = 0;
    SET_CHILDREN(node, leaf ? 0 : 1);
    return node;
}
static uint64_t FN(_position)(Node *node, uint64_t key) {
    uint64_t at = 0;
    while (at < node->count && FN(_compare)(&node->pairs[at].key, &key) < 0) ++at;
    return at;
}
static Pair *FN(_find)(Node *node, uint64_t key) {
    while (node != NULL) {
        uint64_t at = FN(_position)(node, key);
        if (at < node->count && FN(_compare)(&node->pairs[at].key, &key) == 0) return &node->pairs[at];
        if (IS_LEAF(node)) return NULL;
        node = node->child[at];
    }
    return NULL;
}
static void FN(_insert_pair)(Node *node, uint64_t at, Pair pair) {
    memmove(node->pairs + at + 1, node->pairs + at, (node->count - at) * sizeof(Pair));
    node->pairs[at] = pair; ++node->count;
}
static Pair FN(_remove_pair)(Node *node, uint64_t at) {
    Pair pair = node->pairs[at]; --node->count;
    memmove(node->pairs + at, node->pairs + at + 1, (node->count - at) * sizeof(Pair));
    return pair;
}
static void FN(_split)(Node *parent, uint64_t at) {
    Node *left = parent->child[at], *right = FN(_node_new)(IS_LEAF(left));
    Pair median = left->pairs[MIN_PAIRS];
    memcpy(right->pairs, left->pairs + MIN_PAIRS + 1, MIN_PAIRS * sizeof(Pair));
    right->count = MIN_PAIRS; left->count = MIN_PAIRS;
    if (!IS_LEAF(left)) {
        memcpy(right->child, left->child + MIN_PAIRS + 1, (MIN_PAIRS + 1) * sizeof(Node *));
        SET_CHILDREN(left, MIN_PAIRS + 1); SET_CHILDREN(right, MIN_PAIRS + 1);
    }
    uint64_t children = CHILDREN(parent);
    memmove(parent->child + at + 2, parent->child + at + 1, (children - at - 1) * sizeof(Node *));
    parent->child[at + 1] = right; FN(_insert_pair)(parent, at, median); SET_CHILDREN(parent, children + 1);
}
static void FN(_borrow_left)(Node *parent, uint64_t at) {
    Node *child = parent->child[at], *left = parent->child[at - 1];
    STRUCTURAL_COUNT(IS_LEAF(child) ? 0 : 1);
    uint64_t children = CHILDREN(child);
    FN(_insert_pair)(child, 0, parent->pairs[at - 1]);
    parent->pairs[at - 1] = FN(_remove_pair)(left, left->count - 1);
    if (children != 0) {
        uint64_t left_children = CHILDREN(left);
        ++left_children; /* Direct count was just decremented. */
        memmove(child->child + 1, child->child, children * sizeof(Node *));
        child->child[0] = left->child[left_children - 1];
        SET_CHILDREN(child, children + 1); SET_CHILDREN(left, left_children - 1);
    }
}
static void FN(_borrow_right)(Node *parent, uint64_t at) {
    Node *child = parent->child[at], *right = parent->child[at + 1];
    STRUCTURAL_COUNT(IS_LEAF(child) ? 2 : 3);
    uint64_t children = CHILDREN(child), right_children = CHILDREN(right);
    child->pairs[child->count++] = parent->pairs[at];
    parent->pairs[at] = FN(_remove_pair)(right, 0);
    if (children != 0) {
        child->child[children] = right->child[0];
        memmove(right->child, right->child + 1, (right_children - 1) * sizeof(Node *));
        SET_CHILDREN(child, children + 1); SET_CHILDREN(right, right_children - 1);
    }
}
static Node *FN(_merge)(Node *parent, uint64_t at) {
    Node *left = parent->child[at], *right = parent->child[at + 1];
    STRUCTURAL_COUNT(IS_LEAF(left) ? 4 : 5);
    uint64_t children = CHILDREN(parent), left_children = CHILDREN(left), right_children = CHILDREN(right);
    left->pairs[left->count++] = FN(_remove_pair)(parent, at);
    memcpy(left->pairs + left->count, right->pairs, right->count * sizeof(Pair));
    left->count += right->count;
    if (right_children != 0) {
        memcpy(left->child + left_children, right->child, right_children * sizeof(Node *));
        SET_CHILDREN(left, left_children + right_children);
    }
    memmove(parent->child + at + 1, parent->child + at + 2, (children - at - 2) * sizeof(Node *));
    SET_CHILDREN(parent, children - 1); wf_cost_release(right); return left;
}
static Pair FN(_extreme)(Node *node, bool maximum) {
    while (!IS_LEAF(node)) {
        uint64_t at = maximum ? node->count : 0; Node *child = node->child[at];
        if (child->count == MIN_PAIRS) {
            if (at > 0 && node->child[at - 1]->count > MIN_PAIRS) FN(_borrow_left)(node, at);
            else if (at < node->count && node->child[at + 1]->count > MIN_PAIRS) FN(_borrow_right)(node, at);
            else if (at < node->count) child = FN(_merge)(node, at);
            else child = FN(_merge)(node, at - 1);
        }
        node = child;
    }
    return FN(_remove_pair)(node, maximum ? node->count - 1 : 0);
}
static Taken FN(_remove_node)(Node *node, uint64_t key) {
    for (;;) {
        uint64_t at = FN(_position)(node, key);
        if (at < node->count && FN(_compare)(&node->pairs[at].key, &key) == 0) {
            if (IS_LEAF(node)) return (Taken){1, FN(_remove_pair)(node, at)};
            Node *left = node->child[at], *right = node->child[at + 1];
            if (left->count > MIN_PAIRS) {
                STRUCTURAL_COUNT(6);
                Taken result = {1, node->pairs[at]}; node->pairs[at] = FN(_extreme)(left, true); return result;
            }
            if (right->count > MIN_PAIRS) {
                STRUCTURAL_COUNT(7);
                Taken result = {1, node->pairs[at]}; node->pairs[at] = FN(_extreme)(right, false); return result;
            }
            node = FN(_merge)(node, at); continue;
        }
        if (IS_LEAF(node)) return FN(_absent)();
        Node *child = node->child[at];
        if (child->count == MIN_PAIRS) {
            if (at > 0 && node->child[at - 1]->count > MIN_PAIRS) FN(_borrow_left)(node, at);
            else if (at < node->count && node->child[at + 1]->count > MIN_PAIRS) FN(_borrow_right)(node, at);
            else if (at < node->count) child = FN(_merge)(node, at);
            else child = FN(_merge)(node, at - 1);
        }
        node = child;
    }
}
static void FN(_each_node)(Node *node, Digest *digest) {
    if (node == NULL) return;
    for (uint64_t at = 0; at < node->count; ++at) {
        if (!IS_LEAF(node)) FN(_each_node)(node->child[at], digest);
        FN(_visit)(digest, &node->pairs[at].key, &node->pairs[at].value);
    }
    if (!IS_LEAF(node)) FN(_each_node)(node->child[node->count], digest);
}
static void FN(_range_node)(Node *node, uint64_t lower, uint64_t upper, Digest *digest) {
    if (node == NULL) return;
    uint64_t at = FN(_position)(node, lower);
    if (!IS_LEAF(node)) FN(_range_node)(node->child[at], lower, upper, digest);
    while (at < node->count && FN(_compare)(&node->pairs[at].key, &upper) < 0) {
        FN(_visit)(digest, &node->pairs[at].key, &node->pairs[at].value); ++at;
        if (!IS_LEAF(node)) FN(_range_node)(node->child[at], lower, upper, digest);
    }
}
static void FN(_free_node)(Node *node, Digest *digest) {
    if (node == NULL) return;
    for (uint64_t at = 0; at < CHILDREN(node); ++at) FN(_free_node)(node->child[at], digest);
    for (uint64_t at = 0; at < node->count; ++at) FN(_consume)(digest, node->pairs[at].key, node->pairs[at].value);
    wf_cost_release(node);
}
static PUBLIC Map FN(_new)(void) { return (Map){0}; }
static PUBLIC Put FN(_put)(Map *map, uint64_t key, Value value) {
    Pair pair = {key, value}; Pair *existing = FN(_find)(ROOT(map), key);
    if (existing != NULL) { Put result = FN(_put_outcome)(1, *existing); *existing = pair; return result; }
    if (map->length == CEILING) return FN(_put_outcome)(2, pair);
    if (ROOT(map) == NULL) {
        map->root = FN(_node_new)(true);
    }
    if (map->root->count == MAX_PAIRS) {
        STRUCTURAL_COUNT(9);
        Node *root = FN(_node_new)(false); root->child[0] = map->root; SET_CHILDREN(root, 1); map->root = root; FN(_split)(root, 0);
    }
    Node *node = map->root;
    while (!IS_LEAF(node)) {
        uint64_t at = FN(_position)(node, key);
        if (node->child[at]->count == MAX_PAIRS) {
            STRUCTURAL_COUNT(10);
            FN(_split)(node, at);
            if (FN(_compare)(&node->pairs[at].key, &key) < 0) ++at;
        }
        node = node->child[at];
    }
    FN(_insert_pair)(node, FN(_position)(node, key), pair); ++map->length; return FN(_inserted)();
}
static PUBLIC Taken FN(_remove)(Map *map, uint64_t key) {
    if (ROOT(map) == NULL) return FN(_absent)();
    Taken result = FN(_remove_node)(map->root, key);
    if (result.present) --map->length;
    if (map->root->count == 0) {
        if (!IS_LEAF(map->root)) STRUCTURAL_COUNT(8);
        Node *old = map->root; map->root = IS_LEAF(old) ? NULL : old->child[0]; wf_cost_release(old);
    }
    return result;
}
#endif

#if !defined(SOURCE_CONTROL)
static PUBLIC Lookup FN(_lookup)(Map *map, uint64_t key, Digest *digest) {
    Pair *pair = FN(_find)(ROOT(map), key);
    if (pair == NULL) { Lookup result; result.present = 0; return result; }
    return (Lookup){1, FN(_observe)(digest, &pair->key, &pair->value)};
}
static PUBLIC Lookup FN(_edit)(Map *map, uint64_t key, Digest *digest) {
    Pair *pair = FN(_find)(ROOT(map), key);
    if (pair == NULL) { Lookup result; result.present = 0; return result; }
    return (Lookup){1, FN(_edit_value)(digest, &pair->key, &pair->value)};
}
static PUBLIC void FN(_each)(Map *map, Digest *digest) { FN(_each_node)(ROOT(map), digest); }
static PUBLIC void FN(_range)(Map *map, uint64_t lower, uint64_t upper, Digest *digest) {
    if (FN(_compare)(&lower, &upper) < 0) FN(_range_node)(ROOT(map), lower, upper, digest);
}
static PUBLIC void FN(_free)(Map map, Digest *digest) { FN(_free_node)(ROOT(&map), digest); }
#endif
static PUBLIC uint64_t FN(_len)(Map *map) { return map->length; }
static void FN(_put_result)(Put result, Digest *digest) {
    ordered(digest, PUT_STATUS(result));
    if (PUT_STATUS(result) != 0) FN(_consume)(digest, result.pair.key, result.pair.value);
}
static void FN(_taken_result)(Taken result, Digest *digest) {
    ordered(digest, result.present);
    if (result.present) FN(_consume)(digest, result.pair.key, result.pair.value);
}
NOINLINE uint64_t FN(_trace)(uint64_t count, uint64_t rounds, uint64_t seed, uint64_t path) {
    Map map = FN(_new)(); Digest digest = {seed, 0, 0, 0};
    for (uint64_t i = 0; i < count; ++i) {
        uint64_t rank = stream_rank(i, count), key = stream_key(rank);
        FN(_put_result)(FN(_put)(&map, key, FN(_make)(seed + i)), &digest);
    }
    ordered(&digest, FN(_len)(&map));
    for (uint64_t round = 0; round < rounds; ++round) {
        for (uint64_t i = 0; i < count; ++i) {
            uint64_t rank = stream_rank(i, count), key = stream_key(rank);
            if (path == 1) {
                ordered(&digest, LOOKUP_PRESENT(FN(_lookup)(&map, KEY_ARGUMENT(key), &digest)));
                ordered(&digest, LOOKUP_PRESENT(FN(_lookup)(&map, KEY_ARGUMENT(key - 1), &digest)));
            } else if (path == 2) {
                uint64_t next = seed + count + (round * count + i) * 2;
                FN(_put_result)(FN(_put)(&map, key, FN(_make)(next)), &digest);
                ordered(&digest, LOOKUP_PRESENT(FN(_edit)(&map, KEY_ARGUMENT(key), &digest)));
                FN(_taken_result)(FN(_remove)(&map, KEY_ARGUMENT(key)), &digest);
                FN(_put_result)(FN(_put)(&map, key, FN(_make)(next + 1)), &digest);
            } else if (path == 3) {
                uint64_t lower = rank * 2, upper = lower + 32;
                FN(_range)(&map, KEY_ARGUMENT(lower), KEY_ARGUMENT(upper), &digest);
            } else if (path == 4) {
                uint64_t next = seed + count + round * count + i;
                FN(_put_result)(FN(_put)(&map, key, FN(_make)(next)), &digest);
            }
        }
    }
    ordered(&digest, FN(_len)(&map));
    FN(_each)(&map, &digest); FN(_free)(map, &digest); return finish(digest);
}
uint64_t FN(_node_bytes)(void) { return sizeof(Node); }

/* Structural checks are outside all timed traces. The sorted oracle above the
 * driver still owns value semantics; these checks qualify the native baseline's
 * balanced-tree algorithms, including complete deletion and logical refusal. */
static uint64_t FN(_validate_node)(Node *node, uint64_t lower, uint64_t upper,
                                   bool root, uint64_t *pairs) {
#if defined(AVL_CONTROL)
    (void)root;
    if (node == NULL) return 0;
    require(lower < node->pair.key && node->pair.key < upper, "AVL key ordering");
    uint64_t left = FN(_validate_node)(node->left, lower, node->pair.key, false, pairs);
    uint64_t right = FN(_validate_node)(node->right, node->pair.key, upper, false, pairs);
    require(left <= right + 1 && right <= left + 1, "AVL balance");
    require(node->height == 1 + (left > right ? left : right), "AVL stored height");
    ++*pairs; return node->height;
#elif defined(SOURCE_CONTROL)
    if (node == NULL) { require(root, "bundled child presence"); return 0; }
    uint64_t count = node->entries.count, previous = lower, depth = 0;
    require(count <= MAX_PAIRS && count >= (root ? 1 : MIN_PAIRS), "bundled node occupancy");
    require(node->leading.length <= 1, "bundled leading vacancy");
    bool leaf = IS_LEAF(node);
    for (uint64_t i = 0; i < count; ++i) {
        Entry *entry = &node->entries.data[i];
        require(previous < entry->pair.key && entry->pair.key < upper, "bundled key ordering");
        require(entry->after.length == (uint64_t)!leaf, "bundled following child presence");
        if (!leaf) {
            Link *before = i == 0 ? &node->leading : &node->entries.data[i - 1].after;
            uint64_t child_depth = FN(_validate_node)(before->node, previous, entry->pair.key, false, pairs);
            if (i == 0) depth = child_depth;
            require(depth == child_depth, "bundled balanced leaf depth");
        }
        previous = entry->pair.key;
    }
    if (!leaf)
        require(depth == FN(_validate_node)(node->entries.data[count - 1].after.node, previous, upper, false, pairs), "bundled final child depth");
    *pairs += count; return depth + 1;
#else
    if (node == NULL) { require(root, "B-tree child presence"); return 0; }
    require(node->count <= MAX_PAIRS && node->count >= (root ? 1 : MIN_PAIRS), "B-tree occupancy");
    uint64_t previous = lower, depth = 0;
    for (uint64_t i = 0; i < node->count; ++i) {
        require(previous < node->pairs[i].key && node->pairs[i].key < upper, "B-tree key ordering");
        if (!IS_LEAF(node)) {
            uint64_t child_depth = FN(_validate_node)(node->child[i], previous, node->pairs[i].key, false, pairs);
            if (i == 0) depth = child_depth;
            require(depth == child_depth, "B-tree balanced leaf depth");
        }
        previous = node->pairs[i].key;
    }
    if (!IS_LEAF(node)) {
        require(CHILDREN(node) == node->count + 1, "B-tree child prefix");
        require(depth == FN(_validate_node)(node->child[node->count], previous, upper, false, pairs), "B-tree final child depth");
    }
    *pairs += node->count; return depth + 1;
#endif
}
static void FN(_validate)(Map *map) {
    uint64_t pairs = 0;
    (void)FN(_validate_node)(ROOT(map), 0, UINT64_MAX, true, &pairs);
    require(pairs == FN(_len)(map), "tree exact cardinality");
#if defined(SOURCE_CONTROL)
    require(map->root.length == (uint64_t)(pairs != 0), "source root vacancy");
#endif
}
void FN(_audit)(void) {
    Map map = FN(_new)(); Digest digest = {0};
    for (uint64_t i = 0; i < CEILING; ++i) {
        uint64_t key = stream_key(stream_rank(i, CEILING));
        Put inserted = FN(_put)(&map, key, FN(_make)(key));
        require(PUT_STATUS(inserted) == 0, "native audit insertion");
        if (i < 64 || i % 64 == 63) FN(_validate)(&map);
    }
    Put replacement = FN(_put)(&map, 1, FN(_make)(42));
    require(PUT_STATUS(replacement) == 1 && replacement.pair.key == 1, "replacement at logical ceiling");
    Value original = FN(_make)(1);
    require(memcmp(&replacement.pair.value, &original, sizeof original) == 0, "replacement returns original complete payload");
    FN(_consume)(&digest, replacement.pair.key, replacement.pair.value);
    Put refused = FN(_put)(&map, UINT64_C(1000001), FN(_make)(37));
    Value offered = FN(_make)(37);
    require(PUT_STATUS(refused) == 2 && refused.pair.key == UINT64_C(1000001), "full returns offered key");
    require(memcmp(&refused.pair.value, &offered, sizeof offered) == 0, "full returns offered complete payload");
    FN(_consume)(&digest, refused.pair.key, refused.pair.value);
    Digest before = digest;
    FN(_range)(&map, KEY_ARGUMENT(3), KEY_ARGUMENT(3), &digest); FN(_range)(&map, KEY_ARGUMENT(9), KEY_ARGUMENT(3), &digest);
    require(memcmp(&before, &digest, sizeof digest) == 0, "empty and inverted ranges");
    for (uint64_t i = 0; i < CEILING; ++i) {
        /* Alternating extrema repeatedly exercises both directions and root contraction. */
        uint64_t rank = i % 2 ? CEILING - 1 - i / 2 : i / 2;
        uint64_t key = stream_key(rank); Taken taken = FN(_remove)(&map, KEY_ARGUMENT(key));
        Value expected = FN(_make)(key == 1 ? 42 : key);
        require(taken.present && taken.pair.key == key, "native audit complete removal");
        require(memcmp(&taken.pair.value, &expected, sizeof expected) == 0, "removed exact inline payload");
        FN(_consume)(&digest, taken.pair.key, taken.pair.value);
        require(!FN(_remove)(&map, KEY_ARGUMENT(key)).present, "native audit absent removal");
        if (i < 64 || i % 64 == 63) FN(_validate)(&map);
    }
    require(FN(_len)(&map) == 0 && ROOT(&map) == NULL, "contraction to empty");
    Put reused = FN(_put)(&map, 7, FN(_make)(99)); require(PUT_STATUS(reused) == 0, "empty-root reuse");
    FN(_validate)(&map); FN(_free)(map, &digest);
}
#if defined(STRUCTURAL_AUDIT)
void FN(_structural)(void) {
    Map map = FN(_new)(); Digest digest = {0}; memset(structural_counts, 0, sizeof structural_counts);
    for (uint64_t key = 16; key < 152; ++key) {
        Put result = FN(_put)(&map, key, FN(_make)(key)); require(PUT_STATUS(result) == 0, "structural insertion");
    }
    for (uint64_t key = 16; key > 8;) {
        --key; Put result = FN(_put)(&map, key, FN(_make)(key)); require(PUT_STATUS(result) == 0, "structural reverse insertion");
    }
    const uint64_t first[] = {40, 112, 79};
    for (unsigned i = 0; i < 3; ++i) {
        Taken taken = FN(_remove)(&map, KEY_ARGUMENT(first[i])); require(taken.present, "structural initial removal");
        FN(_consume)(&digest, taken.pair.key, taken.pair.value); FN(_validate)(&map);
        if (i == 0) {
            Put result = FN(_put)(&map, 7, FN(_make)(71)); require(PUT_STATUS(result) == 0, "structural split after removal");
            FN(_validate)(&map);
        }
    }
    for (uint64_t step = 0; step < 145; ++step) {
        uint64_t key = 7 + (step * 67) % 145;
        if (key == 40 || key == 112 || key == 79) continue;
        Taken taken = FN(_remove)(&map, KEY_ARGUMENT(key)); require(taken.present, "structural permuted removal");
        FN(_consume)(&digest, taken.pair.key, taken.pair.value); FN(_validate)(&map);
    }
    Put reused = FN(_put)(&map, 17, FN(_make)(99)); require(PUT_STATUS(reused) == 0, "structural empty-root reuse");
    FN(_free)(map, &digest);
    puts("path,count");
    const char *names[] = {"borrow-left-leaf", "borrow-left-internal", "borrow-right-leaf", "borrow-right-internal", "merge-leaf", "merge-internal", "predecessor", "successor", "root-contraction", "root-split", "nonroot-split"};
    for (unsigned i = 0; i < 11; ++i) {
        printf("%s,%" PRIu64 "\n", names[i], structural_counts[i]);
        if (i != 6) require(structural_counts[i] != 0, "structural path was exercised");
    }
}
#endif

#else
#define DECLARE_CONTROL(name) \
    uint64_t name##_trace(uint64_t, uint64_t, uint64_t, uint64_t); \
    uint64_t name##_node_bytes(void); \
    void name##_audit(void)
DECLARE_CONTROL(word_source); DECLARE_CONTROL(record_source);
DECLARE_CONTROL(word_direct); DECLARE_CONTROL(record_direct);
DECLARE_CONTROL(word_avl); DECLARE_CONTROL(record_avl);
#if defined(STRUCTURAL_DRIVER)
void word_source_structural(void);
#endif
extern uint64_t wf_ordered_cost_word_trace(uint64_t, uint64_t, uint64_t, uint64_t);
extern uint64_t wf_ordered_cost_record_trace(uint64_t, uint64_t, uint64_t, uint64_t);
typedef union {
    struct { size_t bytes; uint64_t identity; } value;
    max_align_t alignment;
} AllocationHeader;
typedef struct { size_t requests, requested_bytes, live_nodes, peak_nodes, live_bytes, peak_bytes; } Accounting;
static Accounting accounting;
static size_t required_node_bytes;
static volatile uint64_t observed;
NOINLINE void *wf_cost_allocate(uint64_t bytes) {
    require(bytes == required_node_bytes, "exact per-layout node allocation extent");
    require(bytes <= SIZE_MAX - sizeof(AllocationHeader), "allocation extent");
    AllocationHeader *header = malloc(sizeof *header + (size_t)bytes);
    require(header != NULL, "host allocation failure");
    header->value.bytes = (size_t)bytes; header->value.identity = UINT64_C(0x6f726465726564);
    ++accounting.requests; accounting.requested_bytes += bytes;
    ++accounting.live_nodes; accounting.live_bytes += bytes;
    if (accounting.live_bytes > accounting.peak_bytes) accounting.peak_bytes = accounting.live_bytes;
    if (accounting.live_nodes > accounting.peak_nodes) accounting.peak_nodes = accounting.live_nodes;
    return header + 1;
}
NOINLINE void wf_cost_release(void *pointer) {
    if (pointer == NULL) return;
    AllocationHeader *header = (AllocationHeader *)pointer - 1;
    require(header->value.identity == UINT64_C(0x6f726465726564), "allocation identity");
    require(accounting.live_bytes >= header->value.bytes && accounting.live_nodes > 0, "live allocation accounting");
    accounting.live_bytes -= header->value.bytes; --accounting.live_nodes;
    header->value.identity = 0; free(header);
}
static void reset_accounting(unsigned variant, bool wide) {
    require(accounting.live_bytes == 0 && accounting.live_nodes == 0, "all prior nodes released");
    accounting = (Accounting){0};
    if (variant < 2) required_node_bytes = wide ? record_source_node_bytes() : word_source_node_bytes();
    else if (variant == 2) required_node_bytes = wide ? record_direct_node_bytes() : word_direct_node_bytes();
    else required_node_bytes = wide ? record_avl_node_bytes() : word_avl_node_bytes();
}
static NOINLINE uint64_t run(unsigned variant, bool wide, uint64_t count, uint64_t rounds, uint64_t seed, uint64_t path) {
    uint64_t result;
    if (variant == 0) {
#if defined(NATIVE_ONLY)
        require(false, "WF is not linked into the native-only executable"); result = 0;
#else
        result = wide ? wf_ordered_cost_record_trace(count, rounds, seed, path) : wf_ordered_cost_word_trace(count, rounds, seed, path);
#endif
    }
    else if (variant == 1) result = wide ? record_source_trace(count, rounds, seed, path) : word_source_trace(count, rounds, seed, path);
    else if (variant == 2) result = wide ? record_direct_trace(count, rounds, seed, path) : word_direct_trace(count, rounds, seed, path);
    else result = wide ? record_avl_trace(count, rounds, seed, path) : word_avl_trace(count, rounds, seed, path);
    observed = result; return result;
}

/* Independent oracle: a sorted flat sequence of key/seed records. It neither
 * reuses tree routing nor depends on insertion shape, rebalancing, or fanout. */
typedef struct { uint64_t key, seed; } OraclePair;
static uint64_t oracle_content(OraclePair pair, unsigned words) {
    uint64_t result = pair.key;
    for (unsigned word = 0; word < words; ++word) result = result * UINT64_C(131) + pair.seed + word;
    return result;
}
static size_t oracle_position(const OraclePair *pairs, size_t length, uint64_t key) {
    size_t lower = 0, upper = length;
    while (lower < upper) {
        size_t middle = lower + (upper - lower) / 2;
        if (pairs[middle].key < key) lower = middle + 1; else upper = middle;
    }
    return lower;
}
static void oracle_put(OraclePair *pairs, size_t *length, OraclePair pair, Digest *digest, unsigned words) {
    size_t at = oracle_position(pairs, *length, pair.key);
    if (at < *length && pairs[at].key == pair.key) {
        ordered(digest, 1); consumed(digest, oracle_content(pairs[at], words)); pairs[at] = pair;
    } else if (*length == CEILING) {
        ordered(digest, 2); consumed(digest, oracle_content(pair, words));
    } else {
        memmove(pairs + at + 1, pairs + at, (*length - at) * sizeof *pairs);
        pairs[at] = pair; ++*length; ordered(digest, 0);
    }
}
static uint64_t oracle(bool wide, uint64_t count, uint64_t rounds, uint64_t seed, uint64_t path) {
    OraclePair *pairs = malloc((count + 1) * sizeof *pairs);
    require(pairs != NULL, "oracle allocation"); size_t length = 0;
    unsigned words = wide ? 32 : 1; Digest digest = {seed, 0, 0, 0};
    for (uint64_t i = 0; i < count; ++i)
        oracle_put(pairs, &length, (OraclePair){stream_key(stream_rank(i, count)), seed + i}, &digest, words);
    ordered(&digest, length);
    for (uint64_t round = 0; round < rounds; ++round) {
        for (uint64_t i = 0; i < count; ++i) {
            uint64_t key = stream_key(stream_rank(i, count));
            size_t at = oracle_position(pairs, length, key);
            require(at < length && pairs[at].key == key, "oracle generated hit");
            if (path == 1) {
                ordered(&digest, oracle_content(pairs[at], words)); ordered(&digest, 1); ordered(&digest, 0);
            } else if (path == 2) {
                uint64_t next = seed + count + (round * count + i) * 2;
                oracle_put(pairs, &length, (OraclePair){key, next}, &digest, words);
                ++pairs[at].seed; ordered(&digest, oracle_content(pairs[at], words)); ordered(&digest, 1);
                ordered(&digest, 1); consumed(&digest, oracle_content(pairs[at], words));
                --length; memmove(pairs + at, pairs + at + 1, (length - at) * sizeof *pairs);
                oracle_put(pairs, &length, (OraclePair){key, next + 1}, &digest, words);
            } else if (path == 3) {
                uint64_t lower = key - 1, upper = lower + 32;
                for (size_t p = oracle_position(pairs, length, lower); p < length && pairs[p].key < upper; ++p)
                    ordered(&digest, oracle_content(pairs[p], words));
            } else if (path == 4) {
                uint64_t next = seed + count + round * count + i;
                oracle_put(pairs, &length, (OraclePair){key, next}, &digest, words);
            }
        }
    }
    ordered(&digest, length);
    for (size_t i = 0; i < length; ++i) ordered(&digest, oracle_content(pairs[i], words));
    for (size_t i = 0; i < length; ++i) consumed(&digest, oracle_content(pairs[i], words));
    free(pairs); return finish(digest);
}
static uint64_t nanos(void) {
#if defined(_WIN32)
    LARGE_INTEGER value, frequency;
    require(QueryPerformanceCounter(&value) != 0 && QueryPerformanceFrequency(&frequency) != 0, "monotonic clock");
    return (uint64_t)((long double)value.QuadPart * 1.0e9L / frequency.QuadPart);
#else
    struct timespec value; require(clock_gettime(CLOCK_MONOTONIC, &value) == 0, "monotonic clock");
    return (uint64_t)value.tv_sec * UINT64_C(1000000000) + value.tv_nsec;
#endif
}
static void clock_resolution(void) {
#if defined(_WIN32)
    LARGE_INTEGER frequency;
    require(QueryPerformanceFrequency(&frequency) != 0 && frequency.QuadPart > 0, "clock resolution");
    uint64_t reported = (UINT64_C(1000000000) + (uint64_t)frequency.QuadPart - 1) / (uint64_t)frequency.QuadPart;
#else
    struct timespec resolution;
    require(clock_getres(CLOCK_MONOTONIC, &resolution) == 0, "clock resolution");
    uint64_t reported = (uint64_t)resolution.tv_sec * UINT64_C(1000000000) + resolution.tv_nsec;
#endif
    uint64_t previous = nanos(), grid = 0, minimum = UINT64_MAX;
    for (unsigned read = 0; read < 4096; ++read) {
        uint64_t now = nanos(), delta = now - previous; previous = now;
        if (delta == 0) continue;
        if (delta < minimum) minimum = delta;
        uint64_t left = grid, right = delta;
        while (right != 0) { uint64_t next = left % right; left = right; right = next; }
        grid = left;
    }
    require(grid != 0, "clock advances during resolution observation");
    puts("reported_ns,observed_grid_ns,minimum_delta_ns,quantum_ns");
    printf("%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 "\n", reported, grid, minimum, reported > grid ? reported : grid);
}
static void check(bool native_only) {
    const uint64_t counts[] = {0, 1, 2, 7, 8, 15, 16, 31, 63, 256, 4096};
    const uint64_t seeds[] = {0, 17, UINT64_MAX}; size_t configurations = 0;
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned path = 0; path < 5; ++path)
            for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                for (size_t s = 0; s < sizeof seeds / sizeof seeds[0]; ++s) {
                    uint64_t rounds = path == 0 ? 0 : 2;
                    uint64_t expected = oracle(wide, counts[n], rounds, seeds[s], path);
                    Accounting matched = {0};
                    for (unsigned variant = native_only ? 1 : 0; variant < 4; ++variant) {
                        reset_accounting(variant, wide);
                        uint64_t actual = run(variant, wide, counts[n], rounds, seeds[s], path);
                        if (actual != expected) fprintf(stderr, "wide=%u path=%u count=%" PRIu64 " seed=%" PRIu64 " variant=%u actual=%" PRIu64 " expected=%" PRIu64 "\n", wide, path, counts[n], seeds[s], variant, actual, expected);
                        require(actual == expected, "independent sorted sequence checksum");
                        require(accounting.live_nodes == 0 && accounting.live_bytes == 0, "complete node cleanup");
                        if (variant == 0) matched = accounting;
                        if (variant == 1 && !native_only)
                            require(memcmp(&matched, &accounting, sizeof matched) == 0, "WF/source-shaped node accounting");
                    }
                    ++configurations;
                }
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned variant = 1; variant < 4; ++variant) {
            reset_accounting(variant, wide);
            if (variant == 1) { if (wide) record_source_audit(); else word_source_audit(); }
            else if (variant == 2) { if (wide) record_direct_audit(); else word_direct_audit(); }
            else { if (wide) record_avl_audit(); else word_avl_audit(); }
            require(accounting.live_bytes == 0 && accounting.live_nodes == 0, "audit complete cleanup");
        }
    printf("ordered library costs: %zu configurations, %zu executions and 6 complete native structural audits passed (%s)\n", configurations, configurations * (native_only ? 3 : 4), CONTRACT);
}
static void measure(bool reverse) {
    const uint64_t counts[] = {8, 256, 4096};
    const char *paths[] = {"build-cleanup", "hit-miss", "replace-edit-remove-insert", "range-16", "replace-only"};
    const char *variants[] = {"whitefoot", "source-c", "direct-c", "avl-c"};
    puts("contract,pair_bytes,path,count,variant,sample,rounds,traces,elapsed_ns,checksum,requests,requested_bytes,peak_nodes,peak_bytes");
    for (unsigned wide = 0; wide < 2; ++wide)
        for (unsigned path = 0; path < 5; ++path)
            for (size_t n = 0; n < sizeof counts / sizeof counts[0]; ++n)
                for (unsigned sample = 0; sample < 6; ++sample) {
                    uint64_t count = counts[n], seed = 101 + sample;
                    uint64_t rounds = path == 0 ? 0 : 8192 / count;
                    uint64_t traces = path == 0 ? 4096 / count : 1, expected = 0;
                    for (uint64_t trace = 0; trace < traces; ++trace)
                        expected = expected * UINT64_C(257) + oracle(wide, count, rounds, seed + trace, path);
                    Accounting measured[4];
                    for (unsigned offset = 0; offset < 4; ++offset) {
                        unsigned variant = ((sample % 2) != reverse) ? 3 - offset : offset;
                        reset_accounting(variant, wide); uint64_t before = nanos(), checksum = 0;
                        for (uint64_t trace = 0; trace < traces; ++trace)
                            checksum = checksum * UINT64_C(257) + run(variant, wide, count, rounds, seed + trace, path);
                        uint64_t elapsed = nanos() - before; measured[variant] = accounting;
                        require(checksum == expected && accounting.live_nodes == 0 && accounting.live_bytes == 0, "timed checksum and cleanup");
                        printf("%s,%u,%s,%" PRIu64 ",%s,%u,%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%" PRIu64 ",%zu,%zu,%zu,%zu\n", CONTRACT, wide ? 264 : 16, paths[path], count, variants[variant], sample, rounds, traces, elapsed, checksum, accounting.requests, accounting.requested_bytes, accounting.peak_nodes, accounting.peak_bytes);
                    }
                    require(memcmp(&measured[0], &measured[1], sizeof(Accounting)) == 0, "timed WF/source-shaped allocation agreement");
                }
}
int main(int argc, char **argv) {
    require(argc == 2, "usage: ordered-costs check|native-check|measure|measure-reversed|clock-resolution");
#if defined(STRUCTURAL_DRIVER)
    if (strcmp(argv[1], "structure") == 0) {
        reset_accounting(1, false); word_source_structural();
        require(accounting.live_bytes == 0 && accounting.live_nodes == 0, "structural complete cleanup"); return 0;
    }
#endif
#if defined(NATIVE_ONLY)
    require(strcmp(argv[1], "native-check") == 0, "native-only executable supports native-check");
#endif
    if (strcmp(argv[1], "check") == 0) check(false);
    else if (strcmp(argv[1], "native-check") == 0) check(true);
    else if (strcmp(argv[1], "clock-resolution") == 0) clock_resolution();
    else {
        bool reverse = strcmp(argv[1], "measure-reversed") == 0;
        require(reverse || strcmp(argv[1], "measure") == 0, "usage: ordered-costs check|native-check|measure|measure-reversed|clock-resolution");
        measure(reverse);
    }
    return 0;
}
#endif
