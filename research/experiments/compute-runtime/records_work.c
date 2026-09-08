#include "records_scheduler.h"
#include "records_native.h"

void records_scheduler_chunk(void *opaque, size_t chunk) {
    const RecordWork *work = opaque;
    size_t first = chunk * work->grain;
    size_t count = work->records - first;
    if (count > work->grain) count = work->grain;
    for (size_t i = first; i < first + count; ++i)
        work->output[i] = records_state(work->data + work->offsets[i],
                                       work->offsets[i + 1] - work->offsets[i]);
}
