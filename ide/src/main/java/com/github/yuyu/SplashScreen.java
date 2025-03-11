package com.github.yuyu;

import javafx.animation.KeyFrame;
import javafx.animation.Timeline;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.ProgressBar;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import javafx.util.Duration;

import java.util.Objects;

public class SplashScreen {

    private final Stage splashStage;
    private final ProgressBar progressBar;

    public SplashScreen() {
        splashStage = new Stage();
        splashStage.initStyle(StageStyle.UNDECORATED); // Remove window decorations

        // Load an image
        Image image = new Image(Objects.requireNonNull(getClass().getResource("/com/github/yuyu/img/PEPS-logo.png")).toExternalForm());
        ImageView logo = new ImageView(image);
        // Scale the image to 70% of its original size (30% smaller)
        double scaleFactor = 0.7; // Keep 70% of original size
        double fitWidth = image.getWidth() * scaleFactor;
        double fitHeight = image.getHeight() * scaleFactor;
        logo.setFitWidth(fitWidth);
        logo.setFitHeight(fitHeight);
        logo.setPreserveRatio(true); // Maintain aspect ratio



        double width = image.getWidth();
        double height = image.getHeight();

        // Progress Bar (stretched to full width)
        progressBar = new ProgressBar(0);
        progressBar.setPrefWidth(width); // Make it full width
        progressBar.setStyle("-fx-accent: #0078D7;"); // Optional: Customize color

        // StackPane to overlay the progress bar at the bottom of the image
        StackPane stackPane = new StackPane(logo, progressBar);
        StackPane.setAlignment(progressBar, Pos.BOTTOM_CENTER); // Align the bar at the bottom


        // Set scene size to match the scaled image
        splashStage.setScene(new Scene(stackPane, fitWidth, fitHeight));
    }

    public void show(Runnable onFinish) {
        splashStage.show();

        Timeline timeline = new Timeline(
                new KeyFrame(Duration.millis(100), e -> progressBar.setProgress(progressBar.getProgress() + 0.1))
        );
        timeline.setCycleCount(30); // 30 updates (3 seconds total)
        timeline.setOnFinished(e -> {
            splashStage.close();
            onFinish.run(); // Call main application
        });
        timeline.play();
    }
}
