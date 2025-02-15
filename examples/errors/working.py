"""
This is an example dataflow that uses all the operators.
"""

from datetime import datetime, timedelta, timezone

from bytewax.connectors.stdio import StdOutput
from bytewax.dataflow import Dataflow
from bytewax.testing import TestingInput
from bytewax.tracing import setup_tracing
from bytewax.window import SessionWindow, SystemClockConfig, TumblingWindow

tracer = setup_tracing(log_level="INFO")


def filter_op(x):
    return x % 2 == 0


def filter_map_op(x):
    if x == 0:
        return None
    else:
        return x * 2


def flat_map_op(x):
    return range(x)


def inspect_op(x):
    print(f"Inspect {x}")


def inspect_epoch_op(epoch, x):
    print(f"(epoch {epoch}) Inspect {x}")


def map_op(x):
    return "ALL", [x - 1]


def reduce_op(acc, x):
    return [*acc, x]


def reduce_is_complete(x):
    return True


def folder_builder():
    return {}


def folder_op(acc, x):
    if x[0] in acc:
        acc[x[0]] += 1
    else:
        acc[x[0]] = 1
    return acc


def reduce_window_op(count, event_count):
    return count, event_count


def stateful_map_builder():
    return 0


def stateful_map_op(acc, x):
    return acc, x


def stringify(x):
    return f"{x}"


flow = Dataflow()
flow.input("inp", TestingInput(range(10)))
# Stateless operators
flow.filter(filter_op)
flow.filter_map(filter_map_op)
flow.flat_map(flat_map_op)
flow.inspect(inspect_op)
flow.inspect_epoch(inspect_epoch_op)
flow.map(map_op)
# Stateful operators
flow.reduce("reduce", reduce_op, reduce_is_complete)
cc = SystemClockConfig()
wc = TumblingWindow(
    length=timedelta(seconds=1), align_to=datetime(2023, 1, 1, tzinfo=timezone.utc)
)
flow.fold_window("fold_window", cc, wc, folder_builder, folder_op)
wc = SessionWindow(gap=timedelta(seconds=1))
flow.reduce_window("reduce_window", cc, wc, reduce_window_op)
flow.stateful_map("stateful_map", stateful_map_builder, stateful_map_op)
flow.map(stringify)
flow.output("out", StdOutput())
