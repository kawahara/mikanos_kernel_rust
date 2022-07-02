#include "logger.hpp"

#include "cxx_support.h"

#include <cstddef>
#include <cstdio>
#include <cstring>

namespace {
  LogLevel log_level = kWarn;
}

void SetLogLevel(LogLevel level) {
  log_level = level;
}

int Log(LogLevel level, const char* format, ...) {
   char buf[1024];
   va_list ap;

   va_start(ap, format);
   int res = vsnprintf(buf, sizeof(buf) - 1, format, ap);
   va_end(ap);

   mikanos_log(level, buf, strlen(buf));

   return res;
}
