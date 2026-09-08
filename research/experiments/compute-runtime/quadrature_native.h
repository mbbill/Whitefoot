#ifndef WHITEFOOT_QUADRATURE_NATIVE_H
#define WHITEFOOT_QUADRATURE_NATIVE_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Private recursive benchmark interface. Retire with quadrature. A fork is
 * one application-level pair, not an upstream runtime's internal task count. */
typedef struct { uint64_t nodes, forks, migrated, worker_nodes[4]; } QuadratureObservation;
double quadrature_native_run(unsigned kind, unsigned workers, unsigned spawn_depth,
    double a, double b, double center, double width, double tolerance, unsigned depth);
QuadratureObservation quadrature_native_observation(void);
void quadrature_native_stop(void);
#ifdef __cplusplus
}
#endif
#endif
