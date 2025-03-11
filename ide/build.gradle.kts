plugins {
    id("java")
    id("application")
    id("org.openjfx.javafxplugin") version "0.0.13"
}

val junitVersion = "5.10.0"

java {
    sourceCompatibility = JavaVersion.VERSION_21
    targetCompatibility = JavaVersion.VERSION_21
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("org.controlsfx:controlsfx:11.1.2")
    implementation("org.openjfx:javafx-web:17.0.2")
    implementation("org.apache.commons:commons-text:1.10.0")
    testImplementation("org.junit.jupiter:junit-jupiter-api:$junitVersion")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:$junitVersion")
    implementation(project(":interpreter"))
}

javafx {
    version = "21"
    modules = listOf("javafx.controls", "javafx.fxml", "javafx.web")
}

application {
    mainModule.set("com.github.yuyu")
    mainClass.set("com.github.yuyu.IdeMain")
}
