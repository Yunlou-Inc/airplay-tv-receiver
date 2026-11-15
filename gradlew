#!/bin/sh
APP_HOME=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd -P) || exit 1
CLASSPATH=$APP_HOME/gradle/wrapper/gradle-wrapper.jar
if [ ! -f "$CLASSPATH" ]; then
  echo "ERROR: $CLASSPATH is missing. Restore the reviewed Gradle 8.9 wrapper JAR before building." >&2
  exit 1
fi
if [ -n "$JAVA_HOME" ]; then JAVACMD=$JAVA_HOME/bin/java; else JAVACMD=java; fi
exec "$JAVACMD" -classpath "$CLASSPATH" org.gradle.wrapper.GradleWrapperMain "$@"
