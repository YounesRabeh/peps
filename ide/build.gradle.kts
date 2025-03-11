plugins {
    id("java")
    id("application")
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.apache.commons:commons-text:1.10.0")
}

application {
    // Define the main class for the application.
    mainClass.set("com.github.yuyu.IdeMain")
}