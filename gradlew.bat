@rem Gradle startup script
@echo off
set DIRNAME=%~dp0
if "%DIRNAME%"=="" set DIRNAME=.
set CLASSPATH=%DIRNAME%\gradle\wrapper\gradle-wrapper.jar
if not exist "%CLASSPATH%" (
  echo ERROR: %CLASSPATH% is missing. Restore the reviewed Gradle 8.9 wrapper JAR before building. 1>&2
  exit /b 1
)
if defined JAVA_HOME (
  set JAVA_EXE=%JAVA_HOME%\bin\java.exe
) else (
  set JAVA_EXE=java.exe
)
"%JAVA_EXE%" -classpath "%CLASSPATH%" org.gradle.wrapper.GradleWrapperMain %*
