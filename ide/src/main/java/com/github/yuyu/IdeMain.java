package com.github.yuyu;

import javafx.application.Application;
import javafx.application.Platform;
import javafx.scene.Scene;
import javafx.scene.layout.StackPane;
import javafx.scene.web.WebView;
import javafx.stage.Stage;

import java.util.Objects;

public class IdeMain extends Application {
    @Override
    public void start(Stage primaryStage) {
        SplashScreen splash = new SplashScreen();
        splash.show(() -> Platform.runLater(() -> launchIDE(primaryStage)));

    }

    private void launchIDE(Stage primaryStage) {
        WebView webView = new WebView();

        // Load the HTML file from resources
        String editorHtmlUrl = Objects.requireNonNull(getClass().getResource("/com/github/yuyu/web/editor.html")).toExternalForm();

        webView.getEngine().load(editorHtmlUrl);

        // Allow WebView to expand
        webView.setPrefHeight(Double.MAX_VALUE);
        webView.setPrefWidth(Double.MAX_VALUE);

        StackPane root = new StackPane(webView);
        StackPane.setMargin(webView, new javafx.geometry.Insets(0));

        Scene scene = new Scene(root, 600, 400);

        primaryStage.setTitle("Simple JavaFX Code Editor");
        primaryStage.setScene(scene);
        primaryStage.show();
    }

    public static void main(String[] args) {
        launch(args);
    }
}