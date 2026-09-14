/* Executable evidence for the ordinary linked library. The probe constructs
 * launcher values as a native caller would, then exercises ownership transfer,
 * refusal, ranges and explicit close. View operations enter their private
 * pointer-parameter C bodies; separate WF executions check the LLVM ABI. */
#if !defined(_WIN32)
#ifndef _GNU_SOURCE
#define _GNU_SOURCE
#endif
#endif
#include "ordinary_values.h"
#ifdef NDEBUG
#undef NDEBUG
#endif
#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#if defined(_WIN32)
#include <winsock2.h>
#include <ws2tcpip.h>
#include <direct.h>
#include "windows_runtime.h"
#define wf_chdir _chdir
#else
#include <arpa/inet.h>
#include <fcntl.h>
#include <sys/socket.h>
#include <unistd.h>
#define wf_chdir chdir
#endif
#include "sched/prim.h"

#ifndef WF_COMPLETION_SHUTDOWN
#error "The ordinary-values probe requires its deterministic shutdown observer"
#endif

#if defined(_WIN32)
typedef SOCKET wf_probe_socket;
#define WF_PROBE_SEND SD_SEND
#define WF_PROBE_RECEIVE SD_RECEIVE
#else
typedef int wf_probe_socket;
#define WF_PROBE_SEND SHUT_WR
#define WF_PROBE_RECEIVE SHUT_RD
#endif

static struct {
    wf_prim_wait wait;
    wf_probe_socket socket;
    int direction;
    int armed;
    int paused;
    int resume;
    int finished;
} half_close;

/* Pause immediately before one real host shutdown. The other direction then
 * completes on a different caller: it must neither close the descriptor nor
 * return its credit while this host operation is still outstanding. */
int wf_ordinary_test_shutdown(wf_probe_socket socket, int direction) {
    wf_prim_wait_lock(&half_close.wait);
    if (half_close.armed && half_close.socket == socket
        && half_close.direction == direction) {
        half_close.armed = 0;
        half_close.paused = 1;
        wf_prim_wait_signal(&half_close.wait);
        while (!half_close.resume) wf_prim_wait_sleep(&half_close.wait);
    }
    wf_prim_wait_unlock(&half_close.wait);
    return shutdown(socket, direction);
}

static wf_probe_socket native_socket(const wf_value *value) {
#if defined(_WIN32)
    return (SOCKET)wf__windows_socket_handle((int)value->words[0]);
#else
    return (int)value->words[0];
#endif
}

typedef struct {
    wf_value owner;
    wf_value factory;
    wf_close_result result;
    int send;
} half_close_call;

static void paused_half_close(void *argument) {
    half_close_call *call = argument;
    if (call->send) wf_close_send(&call->result, &call->factory, &call->owner);
    else wf_close_receive(&call->result, &call->factory, &call->owner);
    wf_prim_wait_lock(&half_close.wait);
    half_close.finished = 1;
    wf_prim_wait_signal(&half_close.wait);
    wf_prim_wait_unlock(&half_close.wait);
}

static void check_close(wf_close_result *result) { assert(result->tag == 0); }

static void text_probe(void) {
#if defined(_WIN32)
    static const uint16_t one[] = { 'a', 'b', 'c', 0 };
    static const uint16_t good[] = { 0xd83d, 0xde00, 0 };
    static const uint16_t bad[] = { 0xd800, 0 };
    static const uint16_t empty[] = { 0 };
    const void *words[] = { one, good, bad, empty };
#else
    const void *words[] = { "abc", "\xf0\x9f\x98\x80", "\xed\xa0\x80", "" };
#endif
    wf_value args = {{(uint64_t)(uintptr_t)words, 4, 0, 0}};
    wf_value_result value;
    wf_copy_result copied;
    wf_utf8_result measured;
    unsigned char bytes[16];
    wf_view view = {bytes, sizeof(bytes)};
    assert(wf_args_count(&args) == 4);
    wf_arg_get(&value, &args, 0);
    assert(value.tag == 0);
    memset(bytes, 7, sizeof(bytes));
    wf__body_host_copy_utf8(&copied, &value.value, &view, 2, 3);
    assert(copied.tag == 1 && copied.error.required == 3 && bytes[2] == 7);
    wf__body_host_copy_utf8(&copied, &value.value, &view, 2, 5);
    assert(copied.tag == 0 && copied.value == 5 && memcmp(bytes + 2, "abc", 3) == 0);
    wf_arg_get(&value, &args, 1);
    wf_host_utf8_len(&measured, &value.value);
    assert(measured.tag == 0 && measured.value == 4);
    wf_arg_get(&value, &args, 2);
    wf__body_host_copy_utf8(&copied, &value.value, &view, 0, 16);
    assert(copied.tag == 1 && copied.error.tag == 1);
    wf_arg_get(&value, &args, 3);
    wf_relative_path(&value, &value.value);
    assert(value.tag == 0 && value.value.words[1] == 0);
    wf_arg_get(&value, &args, 4);
    assert(value.tag == 1);
}

static void file_probe(wf_inputs *inputs) {
    static const char filename[] = "ordinary-values.data";
#if defined(_WIN32)
    static const uint16_t component[] = { 'o','r','d','i','n','a','r','y','-','v','a','l','u','e','s','.','d','a','t','a' };
#else
    static const unsigned char component[] = "ordinary-values.data";
#endif
    wf_view name = {(void *)component,
#if defined(_WIN32)
        sizeof(component)
#else
        sizeof(component) - 1
#endif
    };
    unsigned char bytes[4096];
    unsigned char unchanged[sizeof(bytes)];
    wf_view window = {bytes, sizeof(bytes)};
    wf_open_result opened, listing, independent;
    wf_read_result read;
    wf_list_result listed;
    wf_close_result closed;
    wf_value receiving_factory = {{0, 0, 0, 0}};
    wf_value limited_factory = {{1, 0, 0, 0}};
    uint64_t saved = inputs->handles.words[0];
    FILE *fixture = NULL;
#if defined(_WIN32)
    assert(fopen_s(&fixture, filename, "wb") == 0);
#else
    fixture = fopen(filename, "wb");
#endif
    assert(fixture != NULL);
    assert(fwrite("hello", 1, 5, fixture) == 5);
    assert(fclose(fixture) == 0);
    /* This valid component names a regular file, so opening it as a directory
     * fails after taking the sole credit, not at the quota refusal above the
     * host call. The next successful open must consume that same credit. */
    wf__body_open_directory(&listing, &limited_factory, &inputs->cwd, &name, 0, name.length);
    assert(listing.tag == 1 && listing.error.tag != 21 && limited_factory.words[0] == 1);
    wf__body_open_file(&opened, &limited_factory, &inputs->cwd, &name, 0, name.length);
    assert(opened.tag == 0 && limited_factory.words[0] == 0);
    wf_close_read(&closed, &limited_factory, &opened.value);
    check_close(&closed);
    assert(limited_factory.words[0] == 1);
    inputs->handles.words[0] = 0;
    wf__body_open_file(&opened, &inputs->handles, &inputs->cwd, &name, 0, name.length);
    assert(opened.tag == 1 && opened.error.tag == 21 && inputs->handles.words[0] == 0);
    inputs->handles.words[0] = saved;
    wf__body_open_file(&opened, &inputs->handles, &inputs->cwd, &name, 0, name.length);
    assert(opened.tag == 0 && inputs->handles.words[0] == saved - 1);
    memset(bytes, 7, sizeof(bytes));
    wf__body_read_at(&read, &inputs->handles, &opened.value, &window, 0, 2, 9);
    assert(read.tag == 0 && read.value == 7 && bytes[1] == 7 && bytes[7] == 7);
    assert(memcmp(bytes + 2, "hello", 5) == 0);
    memcpy(unchanged, bytes, sizeof(bytes));
    wf__body_read_at(&read, &inputs->handles, &opened.value, &window, 5, 2, 9);
    assert(read.tag == 1 && read.error.tag == 0);
    assert(memcmp(bytes, unchanged, sizeof(bytes)) == 0);
    wf__body_read_at(&read, &inputs->handles, &opened.value, &window, 0, 9, 9);
    assert(read.tag == 0 && read.value == 9);
    wf_close_read(&closed, &receiving_factory, &opened.value);
    check_close(&closed);
    assert(inputs->handles.words[0] == saved - 1 && receiving_factory.words[0] == 1);
    wf_open_directory_source(&listing, &inputs->handles, &inputs->cwd);
    if (listing.tag != 0) {
        assert(listing.error.tag < 28);
        fprintf(stderr, "directory source open: class=%u code=%u origin=%u\n",
                listing.error.tag, listing.error.detail[listing.error.tag].code,
                (unsigned)listing.error.detail[listing.error.tag].origin);
    }
    assert(listing.tag == 0);
    wf_open_directory_source(&independent, &inputs->handles, &inputs->cwd);
    assert(independent.tag == 0);
    wf__body_directory_next(&listed, &listing.value, &window, 11, 11);
    assert(listed.result.tag == 0 && listed.next == 11 && listed.entries == 0);
    wf__body_directory_next(&listed, &listing.value, &window, 3, sizeof(bytes));
    assert(listed.result.tag == 0 && listed.next > 3 && listed.entries > 0);
    do {
        memcpy(unchanged, bytes, sizeof(bytes));
        wf__body_directory_next(&listed, &listing.value, &window, 3, sizeof(bytes));
        assert(listed.next >= 3 && listed.next <= sizeof(bytes));
    } while (listed.result.tag == 0);
    assert(listed.result.error.tag == 0 && listed.next == 3 && listed.entries == 0);
    assert(memcmp(bytes, unchanged, sizeof(bytes)) == 0);
    /* Reopening the same directory must not share the first cursor's EOF. */
    wf__body_directory_next(&listed, &independent.value, &window, 3, sizeof(bytes));
    assert(listed.result.tag == 0 && listed.next > 3 && listed.entries > 0);
    wf_close_directory_source(&closed, &inputs->handles, &independent.value);
    check_close(&closed);
    wf_close_directory_source(&closed, &inputs->handles, &listing.value);
    check_close(&closed);
    assert(remove(filename) == 0);
    /* Transfer the received credit back by actually opening and closing an
     * owner, without comparing the close's factory to its creator. */
    wf_open_directory_source(&listing, &receiving_factory, &inputs->cwd);
    assert(listing.tag == 0 && receiving_factory.words[0] == 0);
    wf_close_directory_source(&closed, &inputs->handles, &listing.value);
    check_close(&closed);
    assert(inputs->handles.words[0] == saved);
}

static uint16_t listener_port(const wf_value *listener) {
    struct sockaddr_in address;
#if defined(_WIN32)
    int length = sizeof(address);
    SOCKET descriptor = (SOCKET)wf__windows_socket_handle((int)listener->words[0]);
#else
    socklen_t length = sizeof(address);
    int descriptor = (int)listener->words[0];
#endif
    memset(&address, 0, sizeof(address));
    assert(getsockname(descriptor, (struct sockaddr *)&address, &length) == 0);
    return ntohs(address.sin_port);
}

static void tcp_probe(wf_inputs *inputs) {
    wf_value address;
    wf_value other_factory = {{0, 0, 0, 0}};
    wf_open_result listener;
    wf_connect_result first_client, second_client;
    wf_accept_result first_server, second_server;
    wf_connection crossed_a, crossed_b;
    wf_close_result closed;
    wf_write_result sent;
    wf_read_result received;
    unsigned char byte = 'x', target = 0;
    wf_view source = {&byte, 1}, destination = {&target, 1};
    uint64_t before = inputs->handles.words[0];
    wf_socket_address_v4(&address, 127, 0, 0, 1, 0);
    wf_tcp_listen(&listener, &inputs->handles, &address);
    if (listener.tag != 0) fprintf(stderr, "listen failed: class=%u code=%u origin=%u\n",
        listener.error.tag, listener.error.detail[listener.error.tag].code,
        listener.error.detail[listener.error.tag].origin);
    assert(listener.tag == 0);
    wf_socket_address_v4(&address, 127, 0, 0, 1, listener_port(&listener.value));
    wf_tcp_connect(&first_client, &inputs->handles, &address);
    assert(first_client.tag == 0);
    wf_tcp_accept(&first_server, &inputs->handles, &listener.value);
    assert(first_server.tag == 0);
    wf_tcp_connect(&second_client, &inputs->handles, &address);
    assert(second_client.tag == 0);
    wf_tcp_accept(&second_server, &inputs->handles, &listener.value);
    assert(second_server.tag == 0);
    assert(inputs->handles.words[0] == before - 5);
    wf__body_send_once(&sent, &first_client.value.send, &source, 0, 1);
    assert(sent.tag == 0 && sent.value == 1);
    wf__body_receive_next(&received, &first_server.value.connection.receive, &destination, 0, 1);
    assert(received.tag == 0 && received.value == 1 && target == 'x');
    crossed_a.receive = first_server.value.connection.receive;
    crossed_a.send = second_server.value.connection.send;
    crossed_b.receive = second_server.value.connection.receive;
    crossed_b.send = first_server.value.connection.send;
    wf_close_receive(&closed, &inputs->handles, &crossed_a.receive); check_close(&closed);
    wf_close_send(&closed, &inputs->handles, &crossed_a.send); check_close(&closed);
    assert(inputs->handles.words[0] == before - 5);
    wf_close_send(&closed, &other_factory, &crossed_b.send); check_close(&closed);
    assert(other_factory.words[0] == 1);
    wf_close_receive(&closed, &inputs->handles, &crossed_b.receive); check_close(&closed);
    assert(inputs->handles.words[0] == before - 4);
    wf_close_receive(&closed, &inputs->handles, &first_client.value.receive); check_close(&closed);
    wf_close_send(&closed, &inputs->handles, &first_client.value.send); check_close(&closed);
    wf_close_send(&closed, &inputs->handles, &second_client.value.send); check_close(&closed);
    wf_close_receive(&closed, &inputs->handles, &second_client.value.receive); check_close(&closed);
    wf_close_listener(&closed, &inputs->handles, &listener.value); check_close(&closed);
    assert(inputs->handles.words[0] + other_factory.words[0] == before);
}

static void concurrent_half_close_probe(wf_inputs *inputs, int send_first) {
    wf_value address;
    wf_open_result listener;
    wf_connect_result client, replacement;
    wf_accept_result server, replacement_server;
    wf_close_result closed;
    wf_write_result sent;
    wf_read_result received;
    wf_prim_thread thread;
    half_close_call call;
    uint64_t before = inputs->handles.words[0];
    uint64_t after_second, descriptor;
    unsigned char byte = 'q', target = 0;
    wf_view source = {&byte, 1}, destination = {&target, 1};

    wf_socket_address_v4(&address, 127, 0, 0, 1, 0);
    wf_tcp_listen(&listener, &inputs->handles, &address);
    assert(listener.tag == 0);
    wf_socket_address_v4(&address, 127, 0, 0, 1, listener_port(&listener.value));
    wf_tcp_connect(&client, &inputs->handles, &address);
    assert(client.tag == 0);
    wf_tcp_accept(&server, &inputs->handles, &listener.value);
    assert(server.tag == 0 && inputs->handles.words[0] == before - 3);
    descriptor = server.value.connection.send.words[0];
    memset(&call, 0, sizeof(call));
    call.send = send_first;
    call.owner = send_first ? server.value.connection.send : server.value.connection.receive;
    wf_prim_wait_lock(&half_close.wait);
    half_close.socket = native_socket(&call.owner);
    half_close.direction = send_first ? WF_PROBE_SEND : WF_PROBE_RECEIVE;
    half_close.armed = 1;
    half_close.paused = 0;
    half_close.resume = 0;
    half_close.finished = 0;
    wf_prim_wait_unlock(&half_close.wait);
    assert(wf_prim_thread_start(&thread, paused_half_close, &call, 1024u * 1024u) == 0);
    wf_prim_wait_lock(&half_close.wait);
    while (!half_close.paused) wf_prim_wait_sleep(&half_close.wait);
    wf_prim_wait_unlock(&half_close.wait);

    if (send_first) wf_close_receive(&closed, &inputs->handles, &server.value.connection.receive);
    else wf_close_send(&closed, &inputs->handles, &server.value.connection.send);
    check_close(&closed);
    after_second = inputs->handles.words[0];
    wf_prim_wait_lock(&half_close.wait);
    half_close.resume = 1;
    wf_prim_wait_signal(&half_close.wait);
    while (!half_close.finished) wf_prim_wait_sleep(&half_close.wait);
    wf_prim_wait_unlock(&half_close.wait);
    check_close(&call.result);
    assert(after_second == before - 3);
    assert(call.factory.words[0] == 1);

    /* The released slot is now reusable. Opening through the credited factory
     * and transferring in both directions must see a fresh connection. */
    wf_tcp_connect(&replacement, &call.factory, &address);
    assert(replacement.tag == 0 && call.factory.words[0] == 0);
    assert(replacement.value.receive.words[0] == descriptor);
    wf_tcp_accept(&replacement_server, &inputs->handles, &listener.value);
    assert(replacement_server.tag == 0);
    wf__body_send_once(&sent, &replacement.value.send, &source, 0, 1);
    assert(sent.tag == 0 && sent.value == 1);
    wf__body_receive_next(&received, &replacement_server.value.connection.receive, &destination, 0, 1);
    assert(received.tag == 0 && received.value == 1 && target == byte);
    target = 0;
    wf__body_send_once(&sent, &replacement_server.value.connection.send, &source, 0, 1);
    assert(sent.tag == 0 && sent.value == 1);
    wf__body_receive_next(&received, &replacement.value.receive, &destination, 0, 1);
    assert(received.tag == 0 && received.value == 1 && target == byte);
    wf_close_send(&closed, &inputs->handles, &replacement.value.send); check_close(&closed);
    wf_close_receive(&closed, &inputs->handles, &replacement.value.receive); check_close(&closed);
    wf_close_receive(&closed, &inputs->handles, &replacement_server.value.connection.receive); check_close(&closed);
    wf_close_send(&closed, &inputs->handles, &replacement_server.value.connection.send); check_close(&closed);
    wf_close_receive(&closed, &inputs->handles, &client.value.receive); check_close(&closed);
    wf_close_send(&closed, &inputs->handles, &client.value.send); check_close(&closed);
    wf_close_listener(&closed, &inputs->handles, &listener.value); check_close(&closed);
    assert(inputs->handles.words[0] == before && call.factory.words[0] == 0);
}

int main(int argc, char **argv) {
    wf_inputs inputs;
    wf_close_result closed;
    assert(argc == 2 && wf_chdir(argv[1]) == 0);
    assert(wf_prim_wait_init(&half_close.wait) == 0);
    text_probe();
    assert(wf__ordinary_inputs(&inputs, 0, NULL));
    assert(inputs.handles.words[0] >= 8);
    file_probe(&inputs);
    tcp_probe(&inputs);
    concurrent_half_close_probe(&inputs, 1);
    concurrent_half_close_probe(&inputs, 0);
    wf_close_directory(&closed, &inputs.handles, &inputs.cwd);
    check_close(&closed);
    wf_prim_wait_destroy(&half_close.wait);
    puts("ordinary linked values: text, range, refusal, file, directory, TCP, crossed halves, concurrent close/reuse, credit transfer passed");
    return 0;
}
