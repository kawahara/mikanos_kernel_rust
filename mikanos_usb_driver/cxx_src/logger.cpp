#include "logger.hpp"

#include <cstddef>
#include <cstdio>

namespace {
  LogLevel log_level = kWarn;
}

void SetLogLevel(LogLevel level) {
  log_level = level;
}

int Log(LogLevel level, const char* format, ...) {
  return 0;
}
